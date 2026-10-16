use super::models::{HealthResponse, ReadyResponse};
use crate::{infrastructure, state::AppState};

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
