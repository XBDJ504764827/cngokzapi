//! 首页统计模型。

use serde::Serialize;

#[derive(Serialize)]
pub struct HomeStats {
    /// 已同步到本站的玩家数。
    pub players: i64,
    /// 地图数。
    pub maps: i64,
    /// 已同步成绩条数。
    pub records: i64,
    /// 在线公开服务器数。
    pub servers_online: i64,
    /// 在线服务器当前总人数。
    pub players_online: i64,
    /// 封禁条目数。
    pub bans: i64,
    /// 统计时间（本地生成，UTC RFC3339）。
    pub updated_at: String,
}
