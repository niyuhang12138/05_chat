use crate::{AppError, AppState, User};
use axum::{extract::State, response::IntoResponse, Extension, Json};

pub(crate) async fn list_chat_user_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let users = state.fetch_chat_user_all(ws_id as _).await?;

    Ok(Json(users))
}
