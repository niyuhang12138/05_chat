mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use anyhow::Context;
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
pub use config::AppConfig;
pub use error::{AppError, ErrorOutput};
use handlers::*;
use middlewares::{set_layer, verify_token};
pub use models::User;
use sqlx::PgPool;
use std::{fmt::Debug, ops::Deref, sync::Arc};
use utils::{DecodingKey, EncodingKey};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pool: PgPool,
}

/// Get the router for the chat application
pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api = Router::new()
        .route("/users", get(list_chat_user_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/{id}",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route("/chat/{id}/message", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        .route("/signup", post(signup_handler))
        .route("/signin", post(signin_handler));
    // .layer();

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layer(app))
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
        })
    }
}

impl Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod test_util {
    use super::*;
    use sqlx::Executor;
    use sqlx_db_tester::TestPg;

    impl AppState {
        pub async fn new_for_test(config: AppConfig) -> Result<(TestPg, Self), AppError> {
            let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
            let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
            // let post = config.server.db_url.rfind('/').expect("invalid db_url");
            // let server_url = &config.server.db_url[..post];
            let (tdb, pool) = get_test_pool(None).await;

            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    ek,
                    dk,
                    pool,
                }),
            };

            Ok((tdb, state))
        }
    }

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://postgres:password@localhost:5432".to_string(),
        };

        let tdb = TestPg::new(url, std::path::Path::new("../migrations"));

        let pool = tdb.get_pool().await;

        // run prepared sql to insert test data
        let sql = include_str!("../fixture/test.sql").split(";");
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }
}
