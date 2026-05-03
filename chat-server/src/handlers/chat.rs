use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chat_core::{Chat, User};

use crate::{
    AppError, AppState,
    error::ErrorOutput,
    models::{CreateChat, UpdateChat},
};
#[utoipa::path(
    get,
    path = "/api/chats",
    responses(
        (status = 200, description = "List of chats", body = Vec<Chat>),
        (status = 403, description = "User signed out", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
/// Get all chats in the chat system.
///
/// - If the user is not signed in, it will return 403.
/// - Otherwise, it will return 200 with a list of chats.
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.fetch_all_chats(user.ws_id).await?;
    Ok((StatusCode::OK, Json(chat)))
}
#[utoipa::path(
    post,
    path = "/api/chats",
    responses(
        (status = 201, description = "Chat created", body = Chat),
        (status = 403, description = "User signed out", body = ErrorOutput),
        (status = 400, description = "Invalid input", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
/// Create a new chat in the chat system.
///
/// - If the user is not signed in, it will return 403.
/// - Otherwise, it will return 201 with a chat.
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(input, user.ws_id).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}
#[utoipa::path(
    get,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id")
    ),
    responses(
        (status = 200, description = "Chat found", body = Chat),
        (status = 404, description = "Chat not found", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
/// Get a chat in the chat system by id.
///
/// - If the chat is not found, it will return 404.
/// - Otherwise, it will return 200 with a chat.
pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id as _).await?;
    match chat {
        Some(chat) => Ok(Json(chat)),
        None => Err(AppError::NotFound(format!("chat id {id}"))),
    }
}
#[utoipa::path(
    put,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id")
    ),
    responses(
        (status = 200, description = "Chat updated", body = UpdateChat),
        (status = 404, description = "Chat not found", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
/// Update a chat in the chat system by id.
///
/// - If the chat is not found, it will return 404.
/// - Otherwise, it will return 200 with a chat.
pub(crate) async fn update_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.update_chat_by_id(id as _, input).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("chat id {id} not found"))),
    }
}
#[utoipa::path(
    delete,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id")
    ),
    responses(
        (status = 200, description = "Chat deleted", body = ()),
        (status = 404, description = "Chat not found", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
/// Delete a chat in the chat system by id.
///
/// - If the chat is not found, it will return 404.
/// - Otherwise, it will return 200 with an empty body.
pub(crate) async fn delete_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = state.delete_chat_by_id(id as _).await?;
    if deleted {
        Ok((StatusCode::OK, Json(())))
    } else {
        Err(AppError::NotFound(format!("chat id {id} not found")))
    }
}
