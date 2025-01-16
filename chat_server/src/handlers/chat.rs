use axum::{response::IntoResponse, Extension};

use crate::User;

pub(crate) async fn list_chat_handler(Extension(user): Extension<User>) -> impl IntoResponse {
    println!("{user:#?}");
    "list_chat_handler"
}
pub(crate) async fn create_chat_handler() -> impl IntoResponse {
    "create_chat_handler"
}
pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "update_chat_handler"
}
pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete_chat_handler"
}
