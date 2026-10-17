//! 地图相关模型，与前端 `features/maps/types.ts` 一一对应。

use serde::Serialize;

/// 地图列表项（三模式 tier 各自独立，0 表示该模式未评级）。
#[derive(Clone, Serialize)]
pub struct MapSummary {
    pub id: i64,
    pub name: String,
    pub tier_kzt: i16,
    pub tier_skz: i16,
    pub tier_vnl: i16,
    pub bonuses: i16,
    /// 地图制作人显示名（来自 kzgo 元数据；可能为空）。
    pub mappers: Vec<String>,
    pub workshop_id: Option<i64>,
    pub filesize: i64,
    pub validated: bool,
}

/// 地图榜条目（同一玩家取最佳成绩）。
#[derive(Clone, Serialize)]
pub struct MapRecordEntry {
    pub rank: u32,
    pub player: String,
    pub steam_id64: String,
    pub avatar: String,
    pub time: String,
    pub points: i64,
    pub date: String,
}

/// 地图详情：地图信息 + 指定模式的成绩榜。
#[derive(Serialize)]
pub struct MapDetail {
    pub map: MapSummary,
    pub mode: String,
    pub records: Vec<MapRecordEntry>,
    /// 本地暂无该图该模式成绩，已入队后台同步。
    pub syncing: bool,
}
