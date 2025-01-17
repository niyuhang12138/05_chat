use super::{Chat, ChatType, ChatUser};
use crate::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, PgPool};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ParamChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    #[serde(default)]
    pub public: bool,
}

impl Chat {
    #[allow(dead_code)]
    pub async fn create(input: &ParamChat, ws_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let chat_type = number_of_people_and_get_chat_type(input, pool).await?;

        let chat = query_as(
            "INSERT INTO chats (ws_id, name, type, members) VALUES ($1, $2, $3, $4) RETURNING id, ws_id, name, type, members, created_at",
        )
        .bind(ws_id as i64)
        .bind(&input.name)
        .bind(chat_type)
        .bind(&input.members)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub async fn update(
        id: u64,
        ws_id: u64,
        input: &ParamChat,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        if Chat::get_by_id(id, ws_id, pool).await?.is_none() {
            return Err(AppError::NotFound("chat not found".to_string()));
        }

        let chat_type = number_of_people_and_get_chat_type(input, pool).await?;

        let chat: Chat = query_as(
            "UPDATE chats SET name = $1, type = $2, members = $3 WHERE id = $4 RETURNING id, ws_id, name, type, members, created_at",
        )
        .bind(&input.name)
        .bind(chat_type)
        .bind(&input.members)
        .bind(id as i64)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub async fn delete(id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let chat: Chat = query_as(
            "DELETE FROM chats WHERE id = $1 RETURNING id, ws_id, name, type, members, created_at",
        )
        .bind(id as i64)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    #[allow(dead_code)]
    pub async fn fetch_all(ws_id: u64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let chats = query_as(
            "SELECT id, ws_id, name, type, members, created_at FROM chats WHERE ws_id = $1",
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;

        Ok(chats)
    }

    #[allow(dead_code)]
    pub async fn get_by_id(id: u64, ws_id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let chat = query_as(
            "SELECT id, ws_id, name, type, members, created_at FROM chats WHERE id = $1 AND ws_id = $2",
        )
        .bind(id as i64)
        .bind(ws_id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(chat)
    }
}

async fn number_of_people_and_get_chat_type(
    input: &ParamChat,
    pool: &PgPool,
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
    let users = ChatUser::fetch_by_ids(&input.members, pool).await?;
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
    use crate::test_util::get_test_pool;

    #[tokio::test]
    async fn create_single_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;

        let input = ParamChat::new("", &[1, 2], false);
        let chat = Chat::create(&input, 1, &pool).await.unwrap();

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Signal);
        assert_eq!(chat.members.len(), 2);
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;

        let input = ParamChat::new("general", &[1, 2, 3], true);
        let chat = Chat::create(&input, 1, &pool).await.unwrap();

        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members.len(), 3);
    }

    #[tokio::test]
    async fn chat_get_by_id() {
        let (_tdb, pool) = get_test_pool(None).await;

        let chat = Chat::get_by_id(1, 1, &pool).await.unwrap().unwrap();
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members.len(), 5);
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;

        let chats = Chat::fetch_all(1, &pool).await.unwrap();
        assert_eq!(chats.len(), 4);
    }

    #[tokio::test]
    async fn update_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;

        let mut input = ParamChat::new("general", &[1, 2, 3], true);
        let chat = Chat::create(&input, 1, &pool).await.unwrap();
        input.name = Some("test".to_string());
        let chat = Chat::update(chat.id as _, 1, &input, &pool).await.unwrap();
        assert_eq!(chat.name, Some("test".to_string()));
    }

    #[tokio::test]
    async fn delete_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;

        let input = ParamChat::new("general", &[1, 2, 3], true);
        let chat = Chat::create(&input, 1, &pool).await.unwrap();
        let chat = Chat::delete(chat.id as _, &pool).await.unwrap();

        if Chat::get_by_id(chat.id as _, chat.ws_id as _, &pool)
            .await
            .unwrap()
            .is_some()
        {
            panic!("chat not deleted")
        }
    }
}
