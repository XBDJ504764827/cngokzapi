//! cngokz-backend 程序入口：初始化日志 → 加载配置 → 构建共享状态 → 装配应用 → 启动监听。

mod app;
mod config;
mod errors;
mod infrastructure;
mod middleware;
mod modules;
mod state;

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
    let state = state::AppState::new(config.clone());
    let app = app::create_app(state);

    let addr = config.bind_addr();
    tracing::info!(
        host = %config.host,
        port = config.port,
        web_dir = %config.web_dir,
        "listening"
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
