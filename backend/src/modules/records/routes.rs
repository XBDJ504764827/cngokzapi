use axum::{routing::get, Router};

use super::handlers;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/records", get(handlers::list))
}
