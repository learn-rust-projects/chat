use std::{fmt, ops::Deref, sync::Arc};

mod config;
mod error;
mod handlers;
mod models;
mod utils;
use anyhow::Context;
use axum::{
    Router,
    routing::{get, patch, post},
};
pub use config::AppConfig;
pub use error::AppError;
use handlers::*;
use models::*;

use crate::utils::{DecodingKey, EncodingKey};
#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(dead_code)]
pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) db: sqlx::PgPool,
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

// 当我调用 state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk).expect("Failed to load public key");
        let ek = EncodingKey::load(&config.auth.sk).expect("Failed to load private key");
        let db = sqlx::PgPool::connect(&config.server.db_url)
            .await
            .context("Failed to connect to database")?;
        Ok(Self {
            inner: Arc::new(AppStateInner { config, dk, ek, db }),
        })
    }
}
pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
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

    let app = Router::new()
        .route("/", axum::routing::get(index_handler))
        .nest("/api", api)
        .with_state(state);
    Ok(app)
}
