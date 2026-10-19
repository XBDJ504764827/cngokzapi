//! 排行榜条目，与前端 `features/leaderboard/types.ts` 一一对应。
//! 数据来自 `player_scores`（上游口径积分 + 本站汇总分，见 PROJECT §9.4）。

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct LeaderboardEntry {
    /// 当前排序下的名次（1 起，跨越分页连续）。
    pub rank: Option<i32>,
    pub player: String,
    pub steam_id64: String,
    /// 头像地址，暂空时前端用占位图（Steam 头像懒加载，见 PROJECT §9.2）。
    pub avatar: String,
    pub maps_completed: i32,
    /// 上游口径总积分（榜单排序依据，见 PROJECT §9.4）。
    pub total_points: i64,
    pub avg_points: i64,
    /// 本地上游成绩条数（随全量导入/按需同步增长）。
    pub records: i64,
    /// 最近游玩日期，本地暂无成绩时为 null。
    pub last_played: Option<String>,
    /// 距今天数；null 表示本地暂无该玩家成绩。
    pub last_played_days_ago: Option<i64>,
}

/// 分页信封。
#[derive(Clone, Serialize)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

#[derive(Clone, Serialize)]
pub struct LeaderboardPage {
    pub data: Vec<LeaderboardEntry>,
    pub pagination: Pagination,
}
