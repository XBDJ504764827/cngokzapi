use axum::{
    extract::{Query, State},
    Json,
};

use super::{dto::LeaderboardQuery, models::LeaderboardEntry, service};
use crate::{errors::AppError, state::AppState};

/// GET /api/v1/leaderboard?mode=KZT —— 公开接口，无需鉴权。
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<LeaderboardQuery>,
) -> Result<Json<Vec<LeaderboardEntry>>, AppError> {
    Ok(Json(service::board(&state, q.mode.as_deref()).await?))
}
