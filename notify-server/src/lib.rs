mod sse;
use anyhow::Result;
use axum::response::{Html, IntoResponse};
use futures_util::StreamExt;
use sqlx::postgres::PgListener;
use sse::sse_handler;
use tracing::info;

// 在编译时把 index.html 文件内容直接“嵌入”到二进制里，作为字符串常量使用。
const INDEX_HTML: &str = include_str!("../index.html");

pub fn get_router() -> axum::Router {
    axum::Router::new()
        .route("/index", axum::routing::get(index_handler))
        .route("/events", axum::routing::get(sse_handler))
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

pub async fn setup_pg_listener() -> Result<()> {
    let mut listener =
        PgListener::connect("postgres://postgres:postgres@localhost:5432/chat").await?;
    listener.listen("chat_updated").await?;
    // chat_message_created
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();
    tokio::spawn(async move {
        while let Some(Ok(message)) = stream.next().await {
            info!("Received message: {:?}", message);
        }
    });

    Ok(())
}
