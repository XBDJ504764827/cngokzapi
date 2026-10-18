//! 上报接收端的请求体（PROJECT §9.6）。

use serde::Deserialize;

/// 原生接口的单条成绩（`POST /api/v1/ingest/records`）。
#[derive(Debug, Clone, Deserialize)]
pub struct IngestItem {
    /// 客户端生成的幂等键（建议 UUID），可省略。
    #[serde(default)]
    pub record_uuid: Option<String>,
    /// SteamID64（17 位数字）。
    pub steam_id64: String,
    /// 玩家昵称，可省略（不覆盖已有昵称）。
    #[serde(default)]
    pub player_name: Option<String>,
    /// 地图名（与 `map_id` 二选一）。
    #[serde(default)]
    pub map_name: Option<String>,
    /// 上游地图 id（与 `map_name` 二选一）。
    #[serde(default)]
    pub map_id: Option<i64>,
    /// KZT / SKZ / VNL。
    pub mode: String,
    #[serde(default)]
    pub stage: i32,
    #[serde(default = "default_tickrate")]
    pub tickrate: i32,
    #[serde(default)]
    pub teleports: i32,
    /// 耗时（毫秒）。
    pub time_ms: i32,
    /// 服务器 id（上游口径，可省略）。
    #[serde(default)]
    pub server_id: Option<i64>,
    /// 成绩时间（RFC3339），省略则取服务端当前时间。
    #[serde(default)]
    pub created_on: Option<String>,
}

fn default_tickrate() -> i32 {
    128
}

/// 插件上报的服务器信息（`POST /api/v1/ingest/servers`）。
/// 装了本站插件的服务器用它上报实时人数/地图/玩家，读取时优先于 Steam 发现的数据。
#[derive(Debug, Clone, Deserialize)]
pub struct IngestServer {
    pub ip: String,
    pub port: i32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub map: Option<String>,
    #[serde(default)]
    pub player_count: Option<i32>,
    #[serde(default)]
    pub max_players: Option<i32>,
    /// 在线玩家昵称。
    #[serde(default)]
    pub players: Option<Vec<String>>,
    /// 模式标记（缺省视为 KZ；用于只展示 KZ 服）。
    #[serde(default)]
    pub game_mode: Option<String>,
    /// 当前地图难度（可选，缺省按地图名从 `maps` 关联）。
    #[serde(default)]
    pub map_tier: Option<i16>,
}

/// GlobalAPI 兼容层的请求体（`POST /api/v2.0/records`，与官方客户端字段一致）。
#[derive(Debug, Clone, Deserialize)]
pub struct CompatRecord {
    /// `STEAM_1:0:12345` 形式。
    pub steam_id: String,
    /// 上游地图 id。
    pub map_id: i64,
    /// GlobalMode：200=KZT / 201=SKZ / 202=VNL（也接受 "KZT" 等字符串）。
    pub mode: serde_json::Value,
    #[serde(default)]
    pub stage: i32,
    #[serde(default = "default_tickrate")]
    pub tickrate: i32,
    #[serde(default)]
    pub teleports: i32,
    /// 耗时（**秒**，浮点，官方客户端口径）。
    pub time: f64,
}
