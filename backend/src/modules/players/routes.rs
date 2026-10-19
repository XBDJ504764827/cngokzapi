use axum::{routing::get, Router};

use super::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/players/resolve", get(handlers::resolve))
        .route("/players/{steam_id}", get(handlers::profile))
        .route("/players/{steam_id}/maps", get(handlers::maps))
}
