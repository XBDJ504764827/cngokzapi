//! 个人资料模型，与前端 `features/profile/types.ts` 一一对应。

use serde::Serialize;

#[derive(Serialize)]
pub struct PlayerProfile {
    pub steam_id64: String,
    pub name: String,
    pub avatar: String,
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
