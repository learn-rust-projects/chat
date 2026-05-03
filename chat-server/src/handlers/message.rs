use axum::{
    Extension, Json,
    extract::{Multipart, Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use chat_core::{Message, User};
use serde::Deserialize;
use tokio::fs;
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::{
    AppError, AppState,
    error::ErrorOutput,
    models::{ChatFile, CreateMessage, ListMessages},
};
#[utoipa::path(
    post,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id"),
    ),
    responses(
        (status = 200, description = "Message created", body = Message),
        (status = 400, description = "Invalid input", body = ErrorOutput),
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
    let msg = state.create_message(input, id as _, user.id as _).await?;

    Ok(Json(msg))
}
#[utoipa::path(
    get,
    path = "/api/chats/{id}/messages",
    params(
        ("id" = u64, Path, description = "Chat id"),
        ("input" = ListMessages, Query, description = "List messages input"),
    ),
    responses(
        (status = 200, description = "List of messages", body = Vec<Message>),
    ),
    security(
        ("token" = [])
    )
)]
/// Get all messages in the chat system by chat id.
///
/// - Otherwise, it will return 200 with a list of messages.
pub(crate) async fn list_message_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Query(input): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    let messages = state.list_messages(input, id as _).await?;
    Ok(Json(messages))
}
#[derive(Deserialize, ToSchema)]
#[allow(unused)]
struct HelloForm {
    name: String,
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    file: String,
}

#[utoipa::path(
    post,
    path = "/api/upload",
    request_body(content = HelloForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "List of uploaded files", body = Vec<String>),
        (status = 400, description = "Invalid input", body = ErrorOutput),
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
    let mut files = vec![];
    let ws_id = user.ws_id;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().map(|s| s.to_string());
        let (Some(filename), data) = (filename, field.bytes().await.unwrap()) else {
            warn!("Failed to read multipart field");
            continue;
        };
        let file = ChatFile::new(ws_id, &filename, &data);
        let path = file.path(&state.config.server.base_dir);

        if path.exists() {
            info!("File {} already exists: {:?}", filename, path);
        } else {
            fs::create_dir_all(path.parent().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::DirectoryNotEmpty,
                    "file path parent is not empty directory",
                )
            })?)
            .await?;
            fs::write(path, data).await?;
        }
        files.push(file.url());
    }
    Ok(Json(files))
}

#[utoipa::path(
    get,
    path = "/api/files/{ws_id}/{path}",
    params(
        ("ws_id" = i64, Path, description = "Workspace id"),
        ("path" = String, Path, description = "File path"),
    ),
    responses(
        (status = 200, description = "File content", body = Vec<u8>),
        (status = 404, description = "File not found", body = ErrorOutput),
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
    info!("Request file: {:?}", path);
    if !path.exists() {
        return Err(AppError::NotFound("File doesn't exist".to_string()));
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    // TODO: streaming
    let body = fs::read(path).await?;
    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.to_string().parse().unwrap());
    Ok((headers, body))
}
