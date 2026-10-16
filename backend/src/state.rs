//! 应用共享状态：Axum `State` 萃取的唯一载体，各模块 handler 按需取用。

use crate::{config::AppConfig, infrastructure};

/// 共享状态。`PgPool` 与 `redis::Client` 均为廉价 Clone，
/// 因此 `AppState` 直接派生 Clone，无需再包 `Arc`。
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    // 首个查库模块落地前暂无人读取，allow 随之消除。
    #[allow(dead_code)]
    pub db: sqlx::PgPool,
    #[allow(dead_code)]
    pub redis: redis::Client,
}

impl AppState {
    /// 懒连接：无 PostgreSQL / Redis 时进程仍可启动，
    /// 涉及数据访问的接口返回 503（见 `errors::AppError`）。
    pub fn new(config: AppConfig) -> Self {
        let db = infrastructure::db::connect_lazy(&config.database_url);
        let redis = infrastructure::cache::connect(&config.redis_url);
        Self { config, db, redis }
    }
}
