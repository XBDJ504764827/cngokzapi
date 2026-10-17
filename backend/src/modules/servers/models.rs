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
    pub online: bool,
    /// 当前地图难度（0 = 未知）。
    pub map_tier: i16,
    /// 实时在线玩家昵称。
    pub player_names: Vec<String>,
}
