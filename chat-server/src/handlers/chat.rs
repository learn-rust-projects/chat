use axum::{Extension, response::IntoResponse};
use tracing::info;

use crate::models::User;

pub(crate) async fn list_chat_handler(Extension(user): Extension<User>) -> impl IntoResponse {
    info!("user: {:?}", user);
    "chat"
}
pub(crate) async fn create_chat_handler() -> impl IntoResponse {
    "create_chat"
}
pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "update_chat"
}
pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete_chat"
}
