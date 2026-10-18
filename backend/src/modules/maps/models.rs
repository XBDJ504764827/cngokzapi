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
    /// 当前模式主关最快成绩（WR）保持者；本地暂无该模式成绩时为 null。
    pub wr_player: Option<String>,
    /// WR 耗时（`M:SS.mmm`）。
    pub wr_time: Option<String>,
    /// WR 记录日期 `YYYY-MM-DD`。
    pub wr_date: Option<String>,
}

/// 地图榜条目（同一玩家在该分类下取最佳成绩）。
#[derive(Clone, Serialize)]
pub struct MapRecordEntry {
    pub rank: u32,
    pub player: String,
    pub steam_id64: String,
    pub avatar: String,
    /// 完成耗时（`M:SS.mmm`）。
    pub time: String,
    pub points: i64,
    /// 存读点次数（PRO 恒为 0）。
    pub teleports: i32,
    /// 记录上传时间（UTC ISO8601）。
    pub created_at: String,
}

/// 单模式的两张榜：PRO = 无存读点，TP = 有存读点。
#[derive(Clone, Serialize)]
pub struct ModeBoard {
    pub mode: String,
    pub wr_pro: Option<MapRecordEntry>,
    pub wr_tp: Option<MapRecordEntry>,
    pub pro: Vec<MapRecordEntry>,
    pub tp: Vec<MapRecordEntry>,
}

/// 地图详情：地图信息 + 三模式各自的 PRO/TP 榜。
#[derive(Serialize)]
pub struct MapDetail {
    pub map: MapSummary,
    pub boards: Vec<ModeBoard>,
    /// 任一模式整图同步尚未完成（已入队，稍后重试可见完整成绩）。
    pub syncing: bool,
}
