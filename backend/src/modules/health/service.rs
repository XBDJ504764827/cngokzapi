use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::{
    models::{HealthResponse, ReadyResponse, SyncHealth, SyncKindHealth},
    repository,
};
use crate::{errors::AppError, infrastructure, state::AppState};

/// 各同步任务族的告警阈值（秒）。
const SYNC_SPECS: &[(&str, i64)] = &[
    ("recent", 120),
    ("record_updates", 600),
    ("bans", 600),
    ("servers", 900),
    ("leaderboard_top", 1800),
    ("leaderboard_full", 43_200),
    ("maps", 172_800),
];

fn group_of(kind: &str, key: &str) -> String {
    if kind == "leaderboard" {
        if key.ends_with(":full") {
            "leaderboard_full".to_string()
        } else {
            "leaderboard_top".to_string()
        }
    } else {
        kind.to_string()
    }
}

/// 存活探针：进程存活即 ok，不依赖 DB/Redis。
pub fn build() -> HealthResponse {
    HealthResponse {
        status: "ok",
        service: "cngokzapi",
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// 就绪探针：同时检查 PG 与 Redis，任一不通即 degraded（不抛错，方便编排系统轮询）。
pub async fn readiness(state: &AppState) -> ReadyResponse {
    let postgres = infrastructure::db::ping(&state.db).await;
    if !postgres {
        tracing::warn!("postgres unreachable");
    }
    let redis = redis_ok(&state.redis).await;
    if !redis {
        tracing::warn!("redis unreachable");
    }
    ReadyResponse {
        status: if postgres && redis {
            "ready"
        } else {
            "degraded"
        },
        postgres,
        redis,
    }
}

async fn redis_ok(client: &redis::Client) -> bool {
    let mut conn = match client.get_multiplexed_tokio_connection().await {
        Ok(conn) => conn,
        Err(_) => return false,
    };
    redis::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .is_ok()
}

/// 同步健康：任务延迟、队列积压、失败/死任务与游标。
pub async fn sync_health(state: &AppState) -> Result<SyncHealth, AppError> {
    let counts = repository::job_counts(&state.db).await?;
    let last_done = repository::last_done(&state.db).await?;
    let cursors = repository::cursors(&state.db).await?;
    let now = Utc::now();

    #[derive(Default)]
    struct Agg {
        pending: i64,
        running: i64,
        failed: i64,
        dead: i64,
    }
    let mut agg: HashMap<String, Agg> = HashMap::new();
    for c in counts {
        let entry = agg.entry(group_of(&c.kind, &c.key)).or_default();
        match c.status.as_str() {
            "pending" => entry.pending += c.count,
            "running" => entry.running += c.count,
            "failed" => entry.failed += c.count,
            "dead" => entry.dead += c.count,
            _ => {}
        }
    }

    let mut last_map: HashMap<String, DateTime<Utc>> = HashMap::new();
    for row in last_done {
        if let Some(t) = row.last_done {
            let group = group_of(&row.kind, &row.key);
            last_map
                .entry(group)
                .and_modify(|current| *current = (*current).max(t))
                .or_insert(t);
        }
    }

    let mut kinds = Vec::with_capacity(SYNC_SPECS.len());
    let mut warnings = Vec::new();
    for (kind, warn_after) in SYNC_SPECS {
        let a = agg.remove(*kind).unwrap_or_default();
        let last_success = last_map.get(*kind).copied();

        // gokz 增量任务用游标时间计算延迟更准确；其余用最后成功时间。
        let cursor_key = match *kind {
            "record_updates" => Some("gokz_records_updated"),
            "bans" => Some("gokz_bans_updated"),
            _ => None,
        };
        let cursor_time = cursor_key
            .and_then(|k| cursors.get(k))
            .and_then(|v| DateTime::parse_from_rfc3339(v).ok())
            .map(|t| t.with_timezone(&Utc));
        let lag = cursor_time
            .or(last_success)
            .map(|t| (now - t).num_seconds().max(0));

        let ok = a.failed == 0 && a.dead == 0 && lag.map(|s| s <= *warn_after).unwrap_or(false);
        if a.failed + a.dead > 0 {
            warnings.push(format!("{kind} 有 {} 个失败/死任务", a.failed + a.dead));
        }
        match lag {
            Some(s) if s > *warn_after => {
                warnings.push(format!("{kind} 同步延迟 {s}s，超过阈值 {warn_after}s"));
            }
            None => warnings.push(format!("{kind} 尚未成功同步过")),
            _ => {}
        }

        kinds.push(SyncKindHealth {
            kind: (*kind).to_string(),
            last_success: last_success.map(|t| t.to_rfc3339()),
            lag_seconds: lag,
            warn_after_seconds: *warn_after,
            pending: a.pending,
            running: a.running,
            failed: a.failed,
            dead: a.dead,
            ok,
        });
    }

    Ok(SyncHealth {
        ok: warnings.is_empty(),
        checked_at: now.to_rfc3339(),
        cursors,
        kinds,
        warnings,
    })
}

#[cfg(test)]
mod sync_tests {
    use super::group_of;

    #[test]
    fn leaderboard_groups_split_full_and_top() {
        assert_eq!(group_of("leaderboard", "KZT"), "leaderboard_top");
        assert_eq!(group_of("leaderboard", "KZT:full"), "leaderboard_full");
        assert_eq!(group_of("recent", "all"), "recent");
    }
}
