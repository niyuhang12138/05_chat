use crate::{
    models::{ChatFile, CreateMessage, DeleteMessage, ListMessage},
    AppError, AppState,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Extension, Json,
};
use chat_core::{Chat, User};
use tokio::fs::{self};
use tracing::{info, warn};

#[utoipa::path(
    post,
    path = "/api/{id}/message",
    responses(
        (status = 200, description = "send message", body = Chat),
    ),
    params(
        ("id" = u64, Path, description = "chat id"),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn send_message_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let message = state.create_message(input, id, user.id as _).await?;
    Ok(Json(message))
}

#[utoipa::path(
    delete,
    path = "/api/{id}/message",
    responses(
        (status = 200, description = "delete message", body = Chat),
    ),
    params(
        ("id" = u64, Path, description = "chat id"),
        DeleteMessage,
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn delete_message_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Query(input): Query<DeleteMessage>,
) -> Result<impl IntoResponse, AppError> {
    let message = state.delete_message(input, id).await?;
    Ok(Json(message))
}

#[utoipa::path(
    get,
    path = "/api/{id}/message",
    responses(
        (status = 200, description = "list messages", body = Vec<Chat>),
    ),
    params(
        ("id" = u64, Path, description = "chat id"),
        ListMessage,
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_message_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Query(input): Query<ListMessage>,
) -> Result<impl IntoResponse, AppError> {
    let messages = state.list_message(input, id).await?;
    Ok(Json(messages))
}

#[utoipa::path(
    get,
    path = "/api/files/{ws_id}/{*path}",
    responses(
        (status = 200, description = "get file", body = Vec<u8>),
    ),
    params(
        ("ws_id" = i64, Path, description = "workspace id"),
        ("path" = String, Path, description = "file path"),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn file_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::NotFound(
            "File doesn't exist or you don't have permission".to_string(),
        ));
    }

    let base_dir = state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::NotFound("File doesn't exist".to_string()));
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    // TODO: streaming
    let body = fs::read(path).await?;
    let mut headers = HeaderMap::new();
    headers.insert("Content-type", mime.to_string().parse().unwrap());
    Ok((headers, body))
}

#[utoipa::path(
    post,
    path = "/api/upload",
    responses(
        (status = 200, description = "upload file", body = Vec<String>),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id as u64;
    let base_dir = &state.config.server.base_dir;
    let mut files = vec![];
    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().map(|name| name.to_string());
        let (Some(filename), Ok(data)) = (filename, field.bytes().await) else {
            warn!("Failed to read multipart field");
            continue;
        };

        let file = ChatFile::new(ws_id, &filename, &data);
        let path = file.path(base_dir);
        if path.exists() {
            info!("File {} already exists: {:?}", filename, path);
        } else {
            fs::create_dir_all(path.parent().expect("file path parent should exists")).await?;
            fs::write(path, data).await?;
        }

        files.push(file.url());
    }

    Ok(Json(files))
}
