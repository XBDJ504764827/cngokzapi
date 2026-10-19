use axum::{
    extract::{Query, State},
    Json,
};

use super::{dto::LeaderboardQuery, models::LeaderboardPage, service};
use crate::{errors::AppError, state::AppState};

/// GET /api/v1/leaderboard?mode=&page=&per_page=&query=&sort=&activity= —— 公开接口。
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<LeaderboardQuery>,
) -> Result<Json<LeaderboardPage>, AppError> {
    Ok(Json(service::board(&state, &q).await?))
}
