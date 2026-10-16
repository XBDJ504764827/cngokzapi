mod api;
mod config;
mod router;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // 日志：默认 info，可用 RUST_LOG 覆盖，例如 RUST_LOG=debug
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cngokz_backend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::AppConfig::from_env();
    let app = router::create_router_with_web_dir(&config.web_dir);

    let addr = config.bind_addr();
    tracing::info!("listening on http://{}, web_dir={}", addr, config.web_dir);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
