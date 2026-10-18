use axum::{routing::post, Router};

use super::handlers;
use crate::state::AppState;

/// 挂在 `/api/v1` 下的原生上报接口。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ingest/records", post(handlers::ingest))
        .route("/ingest/servers", post(handlers::ingest_server))
}

/// 挂在 `/api/v2.0` 下的 GlobalAPI 兼容接口。
pub fn compat_routes() -> Router<AppState> {
    Router::new().route("/records", post(handlers::compat_record))
}
