//! 编排层：地图列表与详情；详情无本地成绩时入队整图同步（PROJECT §9.5）。

use super::{
    dto::MapsQuery,
    models::{MapDetail, MapSummary, ModeBoard},
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

/// 地图详情：返回三模式各自的主关 PRO/TP 榜；任一模式未同步时入队并置 `syncing`。
pub async fn detail(state: &AppState, id: i64) -> Result<MapDetail, AppError> {
    let mut map = repository::get_map(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("地图不存在".to_string()))?;

    let mut syncing = false;
    let mut boards = Vec::with_capacity(mode::MODES.len());
    for m in mode::MODES {
        // 整图同步是否已完成（按 sync_jobs 的 done 状态判断，避免「导入过部分成绩就以为齐了」）
        let job_key = format!("{}|{}", map.name, m);
        let synced = sync::queue::is_done(&state.db, sync::KIND_MAP, &job_key).await?;
        if synced {
            // 7 天 TTL：过期后后台重同步，不阻塞页面（用于修正硬删除/改分）
            sync::queue::enqueue_if_stale(&state.db, sync::KIND_MAP, &job_key, sync::DATA_TTL_SECS)
                .await?;
        } else {
            sync::worker::enqueue_map(state, &map.name, m).await;
            syncing = true;
        }

        let pro = repository::board_records(&state.db, map.id, m, true, TOP_LIMIT).await?;
        let tp = repository::board_records(&state.db, map.id, m, false, TOP_LIMIT).await?;
        boards.push(ModeBoard {
            mode: m.to_string(),
            wr_pro: pro.first().cloned(),
            wr_tp: tp.first().cloned(),
            pro,
            tp,
        });
    }

    // 列表页的 wr_* 字段沿用：取 KZT 的 TP 最快成绩
    if let Some(kzt) = boards.iter().find(|b| b.mode == "KZT") {
        if let Some(wr) = &kzt.wr_tp {
            map.wr_player = Some(wr.player.clone());
            map.wr_time = Some(wr.time.clone());
            map.wr_date = wr.created_at.get(..10).map(str::to_string);
        }
    }

    Ok(MapDetail {
        map,
        boards,
        syncing,
    })
}
