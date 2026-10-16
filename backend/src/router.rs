use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::api::health;

/// 装配全部路由：`/` 为占位首页（第二步会被 web/ 静态页替换），
/// `/api/v1/*` 为版本化 JSON API。
pub fn create_router() -> Router {
    Router::new()
        .route("/", get(root))
        .nest("/api/v1", api_v1_router())
        .layer(TraceLayer::new_for_http())
}

fn api_v1_router() -> Router {
    Router::new().route("/health", get(health::health))
}

async fn root() -> &'static str {
    "cngokzapi backend is running. Web UI coming in step 2."
}
