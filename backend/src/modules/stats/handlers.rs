use axum::{extract::State, Json};

use super::{models::HomeStats, service};
use crate::{errors::AppError, state::AppState};

/// GET /api/v1/stats —— 首页统计（公开）。
pub async fn home(State(state): State<AppState>) -> Result<Json<HomeStats>, AppError> {
    Ok(Json(service::home(&state).await?))
}
