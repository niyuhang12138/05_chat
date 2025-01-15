use std::mem;

use crate::{AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use sqlx::query_as;

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
        let password = hash_password(password)?;
        let user = query_as(
            "INSERT INTO users (email, fullname, password_hash) VALUES ($1, $2, $3) RETURNING id, fullname, email, created_at",
        )
        .bind(email)
        .bind(fullname)
        .bind(password)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Verify email and password
    pub async fn verify(
        email: &str,
        password: &str,
        pool: &sqlx::PgPool,
    ) -> anyhow::Result<Option<Self>, AppError> {
        let user: Option<User> = query_as(
            "SELECT id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid = verify_password(password, &password_hash.unwrap_or_default())?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;

    // Verify password
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();

    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "nyh";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert!(verify_password(password, &password_hash)?);

        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;

        let email = "nyh@qq.com";
        let fullname = "张三";
        let password = "zhangsan123";

        let user = User::create(email, fullname, password, &pool).await?;

        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);
        assert!(user.id > 0);

        let user = User::verify(email, password, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }
}
