use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{
    models::{Chat, ParamChat},
    AppError, AppState, User,
};

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = Chat::fetch_all(user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::OK, Json(chats)))
}

pub(crate) async fn get_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    match Chat::get_by_id(id, user.ws_id as _, &state.pool).await? {
        Some(chat) => Ok(Json(chat)),
        None => Err(AppError::NotFound("chat not found".to_string())),
    }
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<ParamChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(&input, user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn update_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<ParamChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::update(id, user.ws_id as _, &input, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn delete_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::delete(id, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}
