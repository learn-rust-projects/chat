use tracing::warn;

use super::REQUEST_ID_HEADER;
pub async fn set_request_id(
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            let req_id = uuid::Uuid::now_v7().to_string();
            match axum::http::HeaderValue::from_str(&req_id) {
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                Err(_) => {
                    warn!("Failed to set request id");
                    None
                }
            }
        }
    };
    let mut res = next.run(req).await;

    if let Some(id) = id {
        res.headers_mut().insert(REQUEST_ID_HEADER, id);
    }
    res
}
