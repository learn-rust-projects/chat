use std::{ops::Deref, sync::Arc};

mod config;
mod error;
mod handlers;
mod models;
use axum::{
    Router,
    routing::{get, patch, post},
};
pub use config::AppConfig;
pub use error::AppError;
use handlers::*;
pub use models::User;
#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AppStateInner {
    pub(crate) config: AppConfig,
}

// 当我调用 state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}

pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);
    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/{id}",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/{id}/messages", get(list_message_handler));

    Router::new()
        .route("/", axum::routing::get(index_handler))
        .nest("/api", api)
        .with_state(state)
}
