//! 进程内事件总线：同步任务 / 上报写入后发布，SSE 推送给浏览器。
//!
//! 单实例部署用 tokio broadcast 即可；多实例时可在此层换成 Redis Pub/Sub，
//! 调用方（ingest / sync jobs）无需改动。

use chrono::Utc;
use serde::Serialize;
use tokio::sync::broadcast;

/// 推送给前端的事件。
#[derive(Debug, Clone, Serialize)]
pub struct AppEvent {
    /// 订阅主题：`records` / `map:{id}` / `player:{steamid64}` / `bans` / `leaderboard` / `stats`
    pub topic: String,
    /// 事件类型：`created` / `updated` / `invalidated`
    pub kind: String,
    /// 关联 id（地图 id / 玩家 steamid），可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// 事件时间（UTC RFC3339）
    pub at: String,
}

/// 事件总线；`broadcast::Sender` 没有订阅者时发送会被安全忽略。
#[derive(Clone)]
pub struct EventBus(broadcast::Sender<AppEvent>);

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self(tx)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.0.subscribe()
    }

    /// 发布单个主题事件。
    pub fn emit(&self, topic: &str, kind: &str, id: Option<i64>) {
        let event = AppEvent {
            topic: topic.to_string(),
            kind: kind.to_string(),
            id: id.map(|v| v.to_string()),
            at: Utc::now().to_rfc3339(),
        };
        let _ = self.0.send(event);
    }

    /// 一条成绩变化：同时通知记录流、对应地图、对应玩家与首页统计。
    pub fn emit_record(&self, kind: &str, map_id: i64, steam_id: i64) {
        self.emit("records", kind, Some(map_id));
        self.emit(&format!("map:{map_id}"), kind, Some(map_id));
        self.emit(&format!("player:{steam_id}"), kind, Some(steam_id));
        self.emit("stats", "updated", None);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
