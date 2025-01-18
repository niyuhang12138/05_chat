use super::ChatUser;
use crate::{AppError, AppState, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
#[allow(unused)]
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use std::mem;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

impl AppState {
    /// Find a user by email
    pub async fn find_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>, AppError> {
        let user =
            query_as("SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(&self.pool)
                .await?;

        Ok(user)
    }

    /// Create a new user
    // TODO: use transaction for workspace and user creation
    pub async fn create_user(&self, input: &CreateUser) -> anyhow::Result<User, AppError> {
        let user = self.find_user_by_email(&input.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        // check if workspace exists, if not create one
        let ws = match self.find_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&input.workspace, 0).await?,
        };

        let password = hash_password(&input.password)?;
        let user: User = query_as(
            "INSERT INTO users (ws_id, email, fullname, password_hash) VALUES ($1, $2, $3, $4) RETURNING id, ws_id, fullname, email, created_at",
        )
        .bind(ws.id)
        .bind(&input.email)
        .bind(&input.fullname)
        .bind(password)
        .fetch_one(&self.pool)
        .await?;

        if ws.owner_id == 0 {
            ws.update_owner(user.id as u64, &self.pool).await?;
        }

        Ok(user)
    }

    /// Verify email and password
    pub async fn verify_user(&self, input: &SigninUser) -> anyhow::Result<Option<User>, AppError> {
        let user: Option<User> = query_as(
            "SELECT id, ws_id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(&input.email)
        .fetch_optional(&self.pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_password(&input.password, &password_hash.unwrap_or_default())?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn fetch_chat_user_by_ids(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = query_as("SELECT id, fullname, email FROM users WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    #[allow(dead_code)]
    pub async fn fetch_chat_user_all(&self, ws_id: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = query_as("SELECT id, fullname, email FROM users WHERE ws_id = $1")
            .bind(ws_id as i64)
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
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
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
impl CreateUser {
    pub fn new(ws: &str, fullname: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            workspace: ws.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "nyh";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert!(verify_password(password, &password_hash)?);

        Ok(())
    }

    #[tokio::test]
    async fn create_already_exists_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("none", "nyh@qq.com", "张三", "zhangsan123");
        state.create_user(&input).await?;
        let ret = state.create_user(&input).await;
        match ret {
            Err(AppError::EmailAlreadyExists(email)) => {
                assert_eq!(email, input.email);
            }
            _ => panic!("Expecting EmailAlreadyExists error"),
        };

        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("none", "nyh@qq.com", "张三", "zhangsan123");
        let user = state.create_user(&input).await?;
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        assert!(user.id > 0);

        let user = state.find_user_by_email(&input.email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);

        let input = SigninUser::new(&input.email, &input.password);
        let user = state.verify_user(&input).await?;
        assert!(user.is_some());

        Ok(())
    }
}
