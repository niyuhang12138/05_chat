use super::{Chat, ChatType};
use crate::{AppError, AppState};
use serde::{Deserialize, Serialize};
use sqlx::query_as;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ParamChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    #[serde(default)]
    pub public: bool,
}

impl AppState {
    #[allow(dead_code)]
    pub async fn create_chat(&self, input: &ParamChat, ws_id: u64) -> Result<Chat, AppError> {
        let chat_type = self.number_of_people_and_get_chat_type(input).await?;

        let chat = query_as(
            "INSERT INTO chats (ws_id, name, type, members) VALUES ($1, $2, $3, $4) RETURNING id, ws_id, name, type, members, created_at",
        )
        .bind(ws_id as i64)
        .bind(&input.name)
        .bind(chat_type)
        .bind(&input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn update_chat(
        &self,
        id: u64,
        ws_id: u64,
        input: &ParamChat,
    ) -> Result<Chat, AppError> {
        if self.get_chat_by_id(id, ws_id).await?.is_none() {
            return Err(AppError::NotFound("chat not found".to_string()));
        }

        let chat_type = self.number_of_people_and_get_chat_type(input).await?;

        let chat: Chat = query_as(
            "UPDATE chats SET name = $1, type = $2, members = $3 WHERE id = $4 RETURNING id, ws_id, name, type, members, created_at",
        )
        .bind(&input.name)
        .bind(chat_type)
        .bind(&input.members)
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn delete_chat(&self, id: u64) -> Result<Chat, AppError> {
        let chat: Chat = query_as(
            "DELETE FROM chats WHERE id = $1 RETURNING id, ws_id, name, type, members, created_at",
        )
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    #[allow(dead_code)]
    pub async fn fetch_chat_all(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let chats = query_as(
            "SELECT id, ws_id, name, type, members, created_at FROM chats WHERE ws_id = $1",
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    #[allow(dead_code)]
    pub async fn get_chat_by_id(&self, id: u64, ws_id: u64) -> Result<Option<Chat>, AppError> {
        let chat = query_as(
            "SELECT id, ws_id, name, type, members, created_at FROM chats WHERE id = $1 AND ws_id = $2",
        )
        .bind(id as i64)
        .bind(ws_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }

    async fn number_of_people_and_get_chat_type(
        &self,
        input: &ParamChat,
    ) -> Result<ChatType, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::UpdateChatError(
                "Chat must have at least 2 members".to_string(),
            ));
        }
        if len > 8 && input.name.is_none() {
            return Err(AppError::UpdateChatError(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }
        let users = self.fetch_chat_user_by_ids(&input.members).await?;
        if users.len() != len {
            return Err(AppError::UpdateChatError(
                "Some members do not exist".to_string(),
            ));
        }
        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Signal,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };
        Ok(chat_type)
    }
}

#[cfg(test)]
impl ParamChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn create_single_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = ParamChat::new("", &[1, 2], false);
        let chat = state.create_chat(&input, 1).await?;
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Signal);
        assert_eq!(chat.members.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = ParamChat::new("general", &[1, 2, 3], true);
        let chat = state.create_chat(&input, 1).await?;
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn chat_get_by_id() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state.get_chat_by_id(1, 1).await.unwrap().unwrap();
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members.len(), 5);
        Ok(())
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state.fetch_chat_all(1).await?;
        assert_eq!(chats.len(), 4);
        Ok(())
    }

    #[tokio::test]
    async fn update_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let mut input = ParamChat::new("general", &[1, 2, 3], true);
        let chat = state.create_chat(&input, 1).await?;
        input.name = Some("test".to_string());
        let chat = state.update_chat(chat.id as _, 1, &input).await?;
        assert_eq!(chat.name, Some("test".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn delete_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = ParamChat::new("general", &[1, 2, 3], true);
        let chat = state.create_chat(&input, 1).await?;
        let chat = state.delete_chat(chat.id as _).await?;

        if state
            .get_chat_by_id(chat.id as _, chat.ws_id as _)
            .await?
            .is_some()
        {
            panic!("chat not deleted")
        }

        Ok(())
    }
}
