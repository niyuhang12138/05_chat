use crate::{models::Workspace, AppError, AppState, User};
use axum::{extract::State, response::IntoResponse, Extension, Json};

pub(crate) async fn list_chat_user_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.pool;
    let ws_id = user.ws_id;
    let users = Workspace::fetch_all_chat_users(ws_id as _, pool).await?;

    Ok(Json(users))
}
