use axum::{routing::get, Router};

use super::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/leaderboard", get(handlers::list))
}
