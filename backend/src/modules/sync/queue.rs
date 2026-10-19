//! `sync_jobs` 队列：入队、认领、结算（PROJECT §9.5）。

use sqlx::PgPool;

/// 最大尝试次数，超过则标记为 `dead`（再次 `enqueue` 会复活）。
const MAX_ATTEMPTS: i32 = 8;

/// 入队（幂等）：同 `(kind, key)` 已存在则重置为 `pending`；
/// 正在 `running` 的任务不会被重置，避免同一任务并发重复执行。
pub async fn enqueue(pool: &PgPool, kind: &str, key: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO sync_jobs (kind, key, status, attempts, next_run_at, updated_at)
           VALUES ($1, $2, 'pending', 0, now(), now())
           ON CONFLICT (kind, key) DO UPDATE SET
               status     = 'pending',
               attempts   = 0,
               next_run_at = now(),
               last_error = NULL,
               updated_at = now()
           WHERE sync_jobs.status <> 'running'"#,
    )
    .bind(kind)
    .bind(key)
    .execute(pool)
    .await?;
    Ok(())
}

/// 仅在任务不存在或已超过 `stale_secs` 未更新时入队。
/// 用于高频任务（如 10 分钟一次的榜单）触发昂贵的玩家补全，避免重复拉取。
pub async fn enqueue_if_stale(
    pool: &PgPool,
    kind: &str,
    key: &str,
    stale_secs: i64,
) -> Result<(), sqlx::Error> {
    let fresh: Option<(String,)> = sqlx::query_as(
        r#"SELECT status FROM sync_jobs
           WHERE kind = $1 AND key = $2
             AND updated_at > now() - make_interval(secs => $3)"#,
    )
    .bind(kind)
    .bind(key)
    .bind(stale_secs as f64)
    .fetch_optional(pool)
    .await?;
    if fresh.is_none() {
        enqueue(pool, kind, key).await?;
    }
    Ok(())
}

/// 认领一个到期任务，返回 `(id, kind, key, attempts)`。
pub async fn claim(pool: &PgPool) -> Result<Option<(i64, String, String, i32)>, sqlx::Error> {
    sqlx::query_as(
        r#"UPDATE sync_jobs SET status = 'running', updated_at = now()
           WHERE id = (
               SELECT id FROM sync_jobs
               WHERE status IN ('pending', 'failed') AND next_run_at <= now()
               -- 先跑全局任务（maps/servers/bans/leaderboard/recent），
               -- 再跑逐玩家补全（player / player_profile），否则玩家任务会长期饿死全局同步
               ORDER BY (kind IN ('player', 'player_profile')) ASC, next_run_at ASC
               FOR UPDATE SKIP LOCKED
               LIMIT 1
           )
           RETURNING id, kind, key, attempts"#,
    )
    .fetch_optional(pool)
    .await
}

/// 该任务是否已成功执行过（用于判断「整图/玩家是否已同步过」，PROJECT §9.5）。
pub async fn is_done(pool: &PgPool, kind: &str, key: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT status FROM sync_jobs WHERE kind = $1 AND key = $2")
            .bind(kind)
            .bind(key)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0 == "done").unwrap_or(false))
}

/// 执行成功。
pub async fn finish_ok(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE sync_jobs SET status = 'done', updated_at = now(), last_error = NULL WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// 执行失败：指数退避（60s · 3^n，上限 1 小时），超过最大次数置 `dead`。
pub async fn finish_err(
    pool: &PgPool,
    id: i64,
    attempts: i32,
    err: &str,
) -> Result<(), sqlx::Error> {
    let next = attempts + 1;
    if next >= MAX_ATTEMPTS {
        sqlx::query(
            "UPDATE sync_jobs SET status = 'dead', attempts = $2, last_error = $3, updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .bind(next)
        .bind(err)
        .execute(pool)
        .await?;
        return Ok(());
    }
    let backoff_secs = (60i64 * 3i64.pow(next as u32)).min(3600);
    sqlx::query(
        r#"UPDATE sync_jobs
           SET status = 'failed', attempts = $2, last_error = $3,
               next_run_at = now() + make_interval(secs => $4), updated_at = now()
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(next)
    .bind(err)
    .bind(backoff_secs as f64)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    /// 退避序列：60s · 3^n，上限 3600s（与 `finish_err` 保持一致）。
    fn backoff(attempts: u32) -> i64 {
        (60i64 * 3i64.pow(attempts)).min(3600)
    }

    #[test]
    fn backoff_grows_and_caps() {
        assert_eq!(backoff(0), 60);
        assert_eq!(backoff(1), 180);
        assert_eq!(backoff(2), 540);
        assert_eq!(backoff(3), 1620);
        assert_eq!(backoff(9), 3600);
    }
}
