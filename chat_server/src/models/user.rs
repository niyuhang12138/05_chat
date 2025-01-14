use sqlx::query_as;

use crate::{AppError, User};

impl User {
    /// Find a user by email
    pub async fn find_by_email(
        email: &str,
        pool: &sqlx::PgPool,
    ) -> anyhow::Result<Option<Self>, AppError> {
        let user = query_as("SELECT id, fullname, email, created_at FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn create(
        email: &str,
        fullname: &str,
        password: &str,
        pool: &sqlx::PgPool,
    ) -> anyhow::Result<Self, AppError> {
        let user = query_as(
            "INSERT INTO users (email, fullname, password) VALUES ($1, $2, $3) RETURNING id, fullname, email, created_at",
        )
        .bind(email)
        .bind(fullname)
        .bind(password)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}
