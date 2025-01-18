use super::{ChatFile, Message};
use crate::{AppError, AppState};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessage {
    #[serde(default)]
    pub last_id: Option<u64>,
    pub limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMessage {
    pub message_id: u64,
}

#[allow(dead_code)]
impl AppState {
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: u64,
        user_id: u64,
    ) -> Result<Message, AppError> {
        let base_dir = &self.config.server.base_dir;
        // verify content = not empty
        if input.content.is_empty() {
            return Err(AppError::CreateMessageError(
                "content is required".to_string(),
            ));
        }

        // verify files exists
        for s in &input.files {
            let file = ChatFile::from_str(s)?;
            if !file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError(format!(
                    "File {} doesn't exist",
                    s
                )));
            }
        }

        // crate message
        let message: Message = query_as(
          "INSERT INTO messages (chat_id, sender_id, content, files) VALUES ($1, $2, $3, $4) RETURNING id, chat_id, sender_id, content, files, created_at",
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .bind(&input.content)
        .bind(&input.files)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn delete_message(
        &self,
        input: DeleteMessage,
        chat_id: u64,
    ) -> Result<Message, AppError> {
        let message: Message = query_as("DELETE FROM messages WHERE id = $1 AND chat_id = $2 RETURNING id, chat_id, sender_id, content, files, created_at")
            .bind(input.message_id as i64)
            .bind(chat_id as i64)
            .fetch_one(&self.pool)
            .await?;
        Ok(message)
    }

    pub async fn list_message(
        &self,
        input: ListMessage,
        chat_id: u64,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX as _);

        let messages: Vec<Message> = query_as(
            "SELECT id, chat_id, sender_id, content, files, created_at FROM messages WHERE chat_id = $1 AND id < $2 ORDER BY id DESC LIMIT $3",
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(input.limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }
}

#[cfg(test)]
impl CreateMessage {
    pub fn new(content: impl Into<String>, files: Vec<&str>) -> Self {
        Self {
            content: content.into(),
            files: files.into_iter().map(|s| s.into()).collect(),
        }
    }
}

#[cfg(test)]
impl ListMessage {
    pub fn new(last_id: Option<u64>, limit: u64) -> Self {
        Self { last_id, limit }
    }
}

#[cfg(test)]
impl DeleteMessage {
    pub fn new(message_id: u64) -> Self {
        Self { message_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn create_message_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let input = CreateMessage::new("hello", vec![]);
        let message = state.create_message(input, 2, 2).await?;
        assert_eq!(message.content, "hello");
        assert_eq!(message.files.len(), 0);

        let url = upload_dummy_file(&state)?;
        let input = CreateMessage::new("world", vec![&url]);
        let message = state.create_message(input, 2, 2).await?;
        assert_eq!(message.content, "world");
        assert_eq!(message.files.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn delete_message_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let url = upload_dummy_file(&state)?;
        let input = CreateMessage::new("hello", vec![&url]);
        let message = state.create_message(input, 2, 2).await?;
        let input = DeleteMessage::new(message.id as _);
        let message1 = state.delete_message(input, 2).await?;
        assert_eq!(message1, message);
        Ok(())
    }

    #[tokio::test]
    async fn list_message_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = ListMessage::new(None, 6);
        let messages = state.list_message(input, 1).await?;
        assert_eq!(messages.len(), 6);

        let last_id = messages.last().expect("last message should exists").id;
        let input = ListMessage::new(Some(last_id as _), 6);
        let message = state.list_message(input, 1).await?;
        assert_eq!(message.len(), 4);

        Ok(())
    }

    fn upload_dummy_file(state: &AppState) -> Result<String> {
        let file = ChatFile::new(1, "test.txt", b"hello world");
        let path = file.path(&state.config.server.base_dir);
        std::fs::create_dir_all(path.parent().expect("file path parent should exists"))?;
        std::fs::write(&path, b"hello")?;
        Ok(file.url())
    }
}
