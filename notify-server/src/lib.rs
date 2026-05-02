mod config;
mod error;
mod notify;
mod sse;
use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use axum::{
    Router,
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
};
use chat_core::{
    User,
    middlewares::{TokenVerify, verify_token},
    utils::DecodingKey,
};
pub use config::AppConfig;
use dashmap::DashMap;
pub use notify::setup_pg_listener;
use tokio::sync::broadcast;
// 在编译时把 index.html 文件内容直接“嵌入”到二进制里，作为字符串常量使用。
const INDEX_HTML: &str = include_str!("../index.html");

pub type UserMap = Arc<DashMap<i64, broadcast::Sender<Arc<notify::AppEvent>>>>;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    config: AppConfig,
    pub users: UserMap,
    pk: DecodingKey,
}

pub async fn get_router(config: AppConfig) -> anyhow::Result<Router> {
    let state = AppState::new(config);
    setup_pg_listener(state.clone()).await?;
    let app = axum::Router::new()
        .route("/events", axum::routing::get(sse::sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", axum::routing::get(index_handler))
        .with_state(state.clone());
    Ok(app)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TokenVerify for AppState {
    type Error = error::AppError;
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        self.pk.verify(token).map_err(|e| e.into())
    }
}

impl AppState {
    pub fn new(config: config::AppConfig) -> Self {
        let pk = DecodingKey::load(&config.auth.pk).expect("failed to load pk");
        let users = Arc::new(DashMap::new());
        Self {
            inner: Arc::new(AppStateInner { config, users, pk }),
        }
    }
}
