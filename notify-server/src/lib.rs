mod config;
mod error;
mod notify;
mod sse;

use axum::{
    http::Method,
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{verify_token, DecodingKey, TokenVerify, User};
pub use config::NotifyConfig;
use dashmap::DashMap;
use error::AppError;
pub use notify::{setup_pg_listener, AppEvent};
use sse::sse_handler;
use std::{ops::Deref, sync::Arc};
use tokio::sync::broadcast;
use tower_http::cors::{self, CorsLayer};

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: NotifyConfig,
    users: UserMap,
    dk: DecodingKey,
}

const INDEX_HTML: &str = include_str!("../index.html");

pub async fn get_router(config: NotifyConfig) -> anyhow::Result<Router> {
    let state = AppState::new(config);
    setup_pg_listener(state.clone()).await?;

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
            Method::HEAD,
            Method::OPTIONS,
            Method::TRACE,
            Method::CONNECT,
        ])
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);

    let app = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .layer(cors)
        .route("/", get(index_handler))
        .with_state(state);

    Ok(app)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> std::result::Result<User, Self::Error> {
        Ok(self.0.dk.verify(token)?)
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AppState {
    pub fn new(config: NotifyConfig) -> Self {
        let dk = DecodingKey::load(&config.auth.pk).expect("Failed to load public key");
        Self(Arc::new(AppStateInner {
            config,
            dk,
            users: Arc::new(DashMap::default()),
        }))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn notify_server_test() {}
}
