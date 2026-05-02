use notify_server::get_router;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    Layer,
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let console = fmt::Layer::new()
        .with_level(true)
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(console).init();

    let addr = "0.0.0.0:6687";

    let config = notify_server::AppConfig::load().unwrap();
    let app = get_router(config).await?;

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn smoke_test() {
        assert!(std::env::args().next().is_some());
    }
}
