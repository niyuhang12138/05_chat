use sqlx::{query_as, PgPool};

use crate::AppError;

use super::{Chat, ChatType};

pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
}

impl Chat {
    #[allow(dead_code)]
    pub async fn create(input: CreateChat, ws_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let chat = query_as(
            "INSERT INTO chats (ws_id, name, r#type, members) VALUES ($1, $2, $3, $4) RETURNING id, ws_id, name, r#type, members, created_at",
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(ChatType::Group)
        .bind(&input.members)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }
}
