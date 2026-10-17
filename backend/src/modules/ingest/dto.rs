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
