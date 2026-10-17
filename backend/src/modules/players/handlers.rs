use axum::{
    extract::{Path, State},
    Json,
};

use super::{models::PlayerProfile, service};
use crate::{errors::AppError, state::AppState};

/// GET /api/v1/players/{steam_id64} —— 个人资料（公开，首次访问触发后台同步）。
pub async fn profile(
    State(state): State<AppState>,
    Path(steam_id): Path<String>,
) -> Result<Json<PlayerProfile>, AppError> {
    Ok(Json(service::profile(&state, &steam_id).await?))
}
