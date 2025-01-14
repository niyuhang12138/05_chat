use axum::response::IntoResponse;

pub(crate) async fn list_message_handler() -> impl IntoResponse {
    "list_message_handler"
}
