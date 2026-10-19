use axum::{
    extract::{Path, Query, State},
    Json,
};

use super::{
    dto::{PlayerMapsQuery, ResolveQuery},
    models::{PlayerMaps, PlayerProfile, ResolvedPlayer},
    service,
};
use crate::{errors::AppError, state::AppState};

/// GET /api/v1/players/resolve?q=... —— 统一解析 SteamID / SteamID64 / 个人主页链接。
pub async fn resolve(
    State(state): State<AppState>,
    Query(q): Query<ResolveQuery>,
) -> Result<Json<ResolvedPlayer>, AppError> {
    let raw = q.q.unwrap_or_default();
    let steam_id64 = service::resolve(&state, &raw).await?;
    Ok(Json(ResolvedPlayer { steam_id64 }))
}

/// GET /api/v1/players/{steam_id64} —— 个人资料（公开，首次访问触发后台同步）。
pub async fn profile(
    State(state): State<AppState>,
    Path(steam_id): Path<String>,
) -> Result<Json<PlayerProfile>, AppError> {
    Ok(Json(service::profile(&state, &steam_id).await?))
}

/// GET /api/v1/players/{steam_id64}/maps —— 地图进度（已完成 / 未完成，分页）。
pub async fn maps(
    State(state): State<AppState>,
    Path(steam_id): Path<String>,
    Query(query): Query<PlayerMapsQuery>,
) -> Result<Json<PlayerMaps>, AppError> {
    Ok(Json(service::player_maps(&state, &steam_id, &query).await?))
}
