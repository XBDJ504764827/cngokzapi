//! SSE 实时事件推送：`GET /api/v1/stream?topics=records,map:44,leaderboard`。
//!
//! 单实例内存广播（`EventBus`）；断线由浏览器 EventSource 自动重连。

pub mod handlers;
pub mod routes;
