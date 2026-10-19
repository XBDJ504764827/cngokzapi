//! PostgreSQL 连接。业务 SQL 一律写在各模块 `repository.rs`，不在 handler 里拼串。

use sqlx::postgres::{PgPool, PgPoolOptions};

/// 懒连接池：`connect_lazy` 不建连，首次查询才连接；无库时启动不受影响。
pub fn connect_lazy(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(16)
        .connect_lazy(database_url)
        .expect("DATABASE_URL 非法")
}

/// 连通性探针，供 `/api/v1/health/ready` 就绪检查使用。
pub async fn ping(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1").fetch_one(pool).await.is_ok()
}
