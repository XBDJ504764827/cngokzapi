//! 服务器模型，与前端 `features/servers/types.ts` 一一对应。

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ServerEntry {
    pub id: i64,
    pub name: String,
    pub map: String,
    pub ip: String,
    pub port: i32,
    pub player_count: i32,
    pub max_players: i32,
    pub country: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub group_name: Option<String>,
    /// 社区分类：优先上游 `group_name`，缺失时从服务器名推导（见 `category.rs`）。
    pub category: String,
    /// 数据来源：`plugin`（本站插件上报，优先）/ `steam`（Steam Web API）。
    pub source: String,
    pub online: bool,
    /// 当前地图难度（0 = 未知）。
    pub map_tier: i16,
    /// 实时在线玩家昵称。
    pub player_names: Vec<String>,
}
