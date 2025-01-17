mod auth;
mod chat;
mod message;
mod workspace;

pub(crate) use auth::*;
use axum::response::IntoResponse;
pub(crate) use chat::*;
pub(crate) use message::*;
pub(crate) use workspace::list_chat_user_handler;

pub(crate) async fn index_handler() -> impl IntoResponse {
    "index_handler"
}
