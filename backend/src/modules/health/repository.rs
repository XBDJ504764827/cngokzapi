//! 同步健康数据源：读 `sync_jobs` / `sync_state`（本模块唯一 SQL 出口）。

use std::collections::HashMap;

use sqlx::{FromRow, PgPool};

#[derive(FromRow)]
pub struct JobCount {
    pub kind: String,
    pub key: String,
    pub status: String,
    pub count: i64,
}

#[derive(FromRow)]
pub struct JobLastDone {
    pub kind: String,
    pub key: String,
    pub last_done: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn job_counts(pool: &PgPool) -> Result<Vec<JobCount>, sqlx::Error> {
    sqlx::query_as(
        "SELECT kind, key, status, count(*)::BIGINT AS count \
         FROM sync_jobs GROUP BY kind, key, status",
    )
    .fetch_all(pool)
    .await
}

pub async fn last_done(pool: &PgPool) -> Result<Vec<JobLastDone>, sqlx::Error> {
    sqlx::query_as(
        "SELECT kind, key, max(last_success_at) AS last_done \
         FROM sync_jobs GROUP BY kind, key",
    )
    .fetch_all(pool)
    .await
}

pub async fn cursors(pool: &PgPool) -> Result<HashMap<String, String>, sqlx::Error> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT key, value FROM sync_state ORDER BY key")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().collect())
}
