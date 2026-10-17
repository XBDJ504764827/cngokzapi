//! 编排层：地图列表与详情；详情无本地成绩时入队整图同步（PROJECT §9.5）。

use super::{
    dto::MapsQuery,
    models::{MapDetail, MapSummary},
    repository,
};
use crate::{
    errors::AppError,
    modules::{mode, sync},
    state::AppState,
};

/// 地图详情榜返回条数。
pub const TOP_LIMIT: i64 = 100;

/// 地图列表：支持名称模糊 + 难度 + 模式（缺省按综合 tier）。
pub async fn list(state: &AppState, q: MapsQuery) -> Result<Vec<MapSummary>, AppError> {
    let query = q.query.unwrap_or_default().trim().to_string();
    let tier = q.tier.unwrap_or(0).clamp(0, 8);
    let m = match q.mode.as_deref() {
        None | Some("") => "OVR".to_string(),
        Some(m) => mode::validate(Some(m))?.to_string(),
    };
    Ok(repository::list_maps(&state.db, &query, tier, &m).await?)
}

/// 地图详情；`syncing` 为真表示整图同步尚未完成（已入队，稍后重试可见完整成绩）。
pub async fn detail(state: &AppState, id: i64, m: Option<&str>) -> Result<MapDetail, AppError> {
    let m = mode::validate(m)?;
    let map = repository::get_map(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("地图不存在".to_string()))?;

    // 整图同步是否已完成（按 sync_jobs 的 done 状态判断，避免「导入过部分成绩就以为齐了」）
    let job_key = format!("{}|{}", map.name, m);
    let synced = sync::queue::is_done(&state.db, sync::KIND_MAP, &job_key).await?;
    if !synced {
        sync::worker::enqueue_map(state, &map.name, m).await;
    }

    let records = repository::top_records(&state.db, map.id, m, TOP_LIMIT).await?;

    Ok(MapDetail {
        map,
        mode: m.to_string(),
        records,
        syncing: !synced,
    })
}
