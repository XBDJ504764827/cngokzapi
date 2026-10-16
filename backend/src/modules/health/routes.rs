use axum::{routing::get, Router};

use super::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/health/ready", get(handlers::ready))
}
