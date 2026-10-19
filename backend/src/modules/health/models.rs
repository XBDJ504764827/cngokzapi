use std::collections::HashMap;

use serde::Serialize;

/// 健康检查响应体，字段保持稳定，监控与部署脚本可直接依赖。
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

/// 就绪检查响应体：如实反映依赖状态，不可用只降级不报错。
#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    pub postgres: bool,
    pub redis: bool,
}

/// 单个同步任务族的健康状态。
#[derive(Serialize)]
pub struct SyncKindHealth {
    pub kind: String,
    pub last_success: Option<String>,
    pub lag_seconds: Option<i64>,
    pub warn_after_seconds: i64,
    pub pending: i64,
    pub running: i64,
    pub failed: i64,
    pub dead: i64,
    pub ok: bool,
}

/// 同步健康总览：`ok=false` 或 `warnings` 非空时应当告警。
#[derive(Serialize)]
pub struct SyncHealth {
    pub ok: bool,
    pub checked_at: String,
    pub cursors: HashMap<String, String>,
    pub kinds: Vec<SyncKindHealth>,
    pub warnings: Vec<String>,
}
