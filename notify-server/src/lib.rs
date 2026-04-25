mod sse;
use axum::response::{Html, IntoResponse};
use sse::sse_handler;
pub fn get_router() -> axum::Router {
    axum::Router::new()
        .route("/index", axum::routing::get(index_handler))
        .route("/events", axum::routing::get(sse_handler))
}
// 在编译时把 index.html 文件内容直接“嵌入”到二进制里，作为字符串常量使用。
const INDEX_HTML: &str = include_str!("../index.html");
async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
