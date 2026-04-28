mod auth;
mod request_id;
mod server_time;

use axum::{Router, middleware::from_fn};
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
const REQUEST_ID_HEADER: &str = "x-request-id";
const SERVER_TIME_HEADER: &str = "x-server-time";
pub use auth::verify_token;

use crate::middlewares::{request_id::set_request_id, server_time::ServerTimeLayer};
pub fn set_layer(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
                .layer(TraceLayer::new_for_http()
            .make_span_with(
                DefaultMakeSpan::new().include_headers(true)
            )
            .on_request(
                DefaultOnRequest::new().level(Level::INFO)
            )
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros)
            )
        )
            // on so on for `on_eos`, `on_body_chunk`, and `on_failure`)
                .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
                .layer(from_fn(set_request_id))
                .layer(ServerTimeLayer),
    )
}
