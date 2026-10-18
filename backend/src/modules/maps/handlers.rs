use axum::{
    extract::{Path, Query, State},
    Json,
};

use super::{
    dto::MapsQuery,
    models::{MapDetail, MapSummary},
    service,
};
use crate::{errors::AppError, state::AppState};

/// GET /api/v1/maps —— 地图列表（公开，支持 query/tier/mode 过滤）。
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<MapsQuery>,
) -> Result<Json<Vec<MapSummary>>, AppError> {
    Ok(Json(service::list(&state, q).await?))
}

/// GET /api/v1/maps/{id} —— 地图详情 + 三模式主关 PRO/TP 榜（公开）。
pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<MapDetail>, AppError> {
    Ok(Json(service::detail(&state, id).await?))
}
