//! 个人资料模型，与前端 `features/profile/types.ts` 一一对应。

use serde::Serialize;

#[derive(Serialize)]
pub struct PlayerProfile {
    pub steam_id64: String,
    pub name: String,
    pub avatar: String,
    /// 国家/地区代码（ISO 3166 alpha-2，如 `CN`/`UA`），来自 GOKZ.TOP。
    pub country: Option<String>,
    pub region: Option<String>,
    /// 本地暂无该玩家数据、已入队后台同步；前端可提示"同步中"。
    pub syncing: bool,
    /// 三模式档案（上游口径排名 + 本站汇总分）。
    pub scores: Vec<PlayerScoreEntry>,
    /// 本地已同步的最近成绩。
    pub recent_records: Vec<PlayerRecordEntry>,
}

#[derive(Clone, Serialize)]
pub struct PlayerScoreEntry {
    pub mode: String,
    pub rank: Option<i32>,
    pub points: i64,
    pub points_local: Option<i64>,
    pub rating: Option<f64>,
    pub maps_completed: i32,
    pub wrs_nub: i32,
    pub wrs_pro: i32,
    /// 本地已同步成绩条数。
    pub records: i64,
}

#[derive(Clone, Serialize)]
pub struct PlayerRecordEntry {
    /// 地图 id：用于地图图片与跳转地图详情。
    pub map_id: i64,
    pub map: String,
    pub mode: String,
    pub stage: i32,
    pub time: String,
    pub points: i64,
    pub date: String,
}

/// 解析结果：统一为 SteamID64。
#[derive(Serialize)]
pub struct ResolvedPlayer {
    pub steam_id64: String,
}

/// 「地图进度」条目：已完成 / 未完成地图。
#[derive(Clone, Serialize)]
pub struct PlayerMapEntry {
    pub map_id: i64,
    pub name: String,
    /// 当前模式的难度（0 = 未评级）。
    pub tier: i16,
    /// 已完成主关的最佳耗时（格式化）；未完成时为 None。
    pub best_time: Option<String>,
    /// 该最佳成绩的分数；未完成时为 None。
    pub points: Option<i64>,
    pub completed: bool,
}

/// 分页结果 + 标签页计数。
#[derive(Serialize)]
pub struct PlayerMaps {
    pub mode: String,
    pub status: String,
    pub page: i64,
    pub per_page: i64,
    /// 当前状态 + 名称搜索下的总数。
    pub total: i64,
    /// 该模式已完成地图数（不受搜索影响，供标签页展示）。
    pub completed_total: i64,
    /// 该模式地图总数。
    pub maps_total: i64,
    pub data: Vec<PlayerMapEntry>,
}
