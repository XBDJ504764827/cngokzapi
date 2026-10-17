use axum::{routing::get, Router};

use super::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/maps", get(handlers::list))
        .route("/maps/{id}", get(handlers::detail))
}
