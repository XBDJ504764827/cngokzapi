//! 应用共享状态：Axum `State` 萃取的唯一载体，各模块 handler 按需取用。

use std::sync::Arc;

use crate::{config::AppConfig, events::EventBus, infrastructure, sources::SourceClient};

/// 共享状态。`PgPool` 与 `redis::Client` 均为廉价 Clone，
/// `SourceClient` 内含限速闸门（非 Clone）故用 `Arc` 共享。
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
    /// 外部数据源客户端（限速 + 重试），见 PROJECT §9.9。
    pub source: Arc<SourceClient>,
    /// 进程内事件总线（SSE 推送给前端）。
    pub events: EventBus,
}

impl AppState {
    /// 懒连接：无 PostgreSQL / Redis 时进程仍可启动，
    /// 涉及数据访问的接口返回 503（见 `errors::AppError`）。
    pub fn new(config: AppConfig) -> Self {
        let db = infrastructure::db::connect_lazy(&config.database_url);
        let redis = infrastructure::cache::connect(&config.redis_url);
        Self {
            config,
            db,
            redis,
            source: Arc::new(SourceClient::new()),
            events: EventBus::new(),
        }
    }
}
