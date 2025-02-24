mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod openapi;

use anyhow::Context;
use axum::{
    http::Method,
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use chat_core::{set_layer, verify_token, DecodingKey, EncodingKey, TokenVerify};
pub use config::ChatConfig;
pub use error::{AppError, ErrorOutput};
use handlers::*;
use middlewares::verify_chat;
pub use models::ParamChat;
use openapi::OpenApiRouter;
use sqlx::PgPool;
use std::{fmt::Debug, ops::Deref, sync::Arc};
use tokio::fs;
use tower_http::cors;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<chat_core::User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

#[allow(unused)]
pub struct AppStateInner {
    pub(crate) config: ChatConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pool: PgPool,
}

/// Get the router for the chat application
pub async fn get_router(state: AppState) -> Result<Router, AppError> {
    let chat: Router<AppState> = Router::new()
        .route(
            "/{id}",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route(
            "/{id}/message",
            get(list_message_handler)
                .delete(delete_message_handler)
                .post(send_message_handler),
        )
        .layer(from_fn_with_state(state.clone(), verify_chat))
        .route("/", get(list_chat_handler).post(create_chat_handler));

    let cors = cors::CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);

    let api = Router::new()
        .route("/users", get(list_chat_user_handler))
        .nest("/chats", chat)
        .route("/upload", post(upload_handler))
        .route("/files/{ws_id}/{*path}", get(file_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/signup", post(signup_handler))
        .route("/signin", post(signin_handler))
        .layer(cors);

    let app = Router::new()
        .openapi()
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
    pub async fn try_new(config: ChatConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir)
            .await
            .context("create base_dir failed")?;
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

#[cfg(feature = "test-util")]
mod test_util {
    use super::*;
    use sqlx::Executor;
    use sqlx_db_tester::TestPg;

    impl AppState {
        pub async fn new_for_test() -> Result<(TestPg, Self), AppError> {
            let config = ChatConfig::load()?;
            let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
            let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
            let post = config.server.db_url.rfind('/').expect("invalid db_url");
            let server_url = &config.server.db_url[..post];
            let (tdb, pool) = get_test_pool(Some(server_url)).await;
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
            None => "postgres://postgres:postgres@localhost:5432".to_string(),
        };
        let tdb = TestPg::new(url, std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test dat
        let sql = include_str!("../fixture/test.sql").split(';');
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
