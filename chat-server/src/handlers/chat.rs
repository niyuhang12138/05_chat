use crate::{models::ParamChat, AppError, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chat_core::{Chat, User};

#[utoipa::path(
    get,
    path = "/api/chats",
    responses(
        (status = 200, description = "list chats", body = Vec<Chat>)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.fetch_chat_all(user.ws_id as _, user.id as _).await?;
    Ok((StatusCode::OK, Json(chats)))
}

#[utoipa::path(
    get,
    path = "/api/chats/{id}",
    responses(
        (status = 200, description = "get chat", body = Chat),
    ),
    params(
        ("id" = u64, Path, description = "chat id"),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn get_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    match state.get_chat_by_id(id, user.ws_id as _).await? {
        Some(chat) => Ok(Json(chat)),
        None => Err(AppError::NotFound("chat not found".to_string())),
    }
}

#[utoipa::path(
    post,
    path = "/api/chats",
    responses(
        (status = 200, description = "create chat", body = Chat)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<ParamChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state
        .create_chat(&input, user.id as _, user.ws_id as _)
        .await?;
    Ok((StatusCode::OK, Json(chat)))
}

#[utoipa::path(
    patch,
    path = "/api/chats/{id}",
    responses(
        (status = 200, description = "update chat", body = Chat)
    ),
    params(
        ("id" = u64, Path, description = "chat id"),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn update_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<ParamChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state
        .update_chat(id, user.ws_id as _, &input, user.id as _)
        .await?;
    Ok((StatusCode::OK, Json(chat)))
}

#[utoipa::path(
    delete,
    path = "/api/chats/{id}",
    responses(
        (status = 200, description = "delete chat", body = Chat)
    ),
    params(
        ("id" = u64, Path, description = "chat id"),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn delete_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.delete_chat(id).await?;
    Ok((StatusCode::OK, Json(chat)))
}
