use axum::response::IntoResponse;

pub(crate) async fn signin_handler() -> impl IntoResponse {
    "signin_handler"
}

pub(crate) async fn signup_handler() -> impl IntoResponse {
    "signup_handler"
}
