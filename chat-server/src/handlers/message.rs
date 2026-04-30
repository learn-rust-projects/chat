use axum::{
    Extension, Json,
    extract::{Multipart, Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use chat_core::User;
use tokio::fs;
use tracing::{info, warn};

use crate::{
    AppError, AppState,
    models::{ChatFile, CreateMessage, ListMessages},
};

pub(crate) async fn send_message_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let msg = state.create_message(input, id as _, user.id as _).await?;

    Ok(Json(msg))
}
pub(crate) async fn list_message_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Query(input): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    let messages = state.list_messages(input, id as _).await?;
    Ok(Json(messages))
}
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
    headers.insert("content-type", mime.to_string().parse().unwrap());
    Ok((headers, body))
}
