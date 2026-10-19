use axum::{extract::State, Json};

use super::{
    models::{HealthResponse, ReadyResponse, SyncHealth},
    service,
};
use crate::{errors::AppError, state::AppState};

/// GET /api/v1/health —— 无需鉴权。
pub async fn health() -> Json<HealthResponse> {
    Json(service::build())
}

/// GET /api/v1/health/ready —— 无需鉴权，返回 PG/Redis 连通性。
pub async fn ready(State(state): State<AppState>) -> Json<ReadyResponse> {
    Json(service::readiness(&state).await)
}

/// GET /api/v1/health/sync —— 同步任务延迟 / 积压 / 失败告警。
pub async fn sync_health(State(state): State<AppState>) -> Result<Json<SyncHealth>, AppError> {
    Ok(Json(service::sync_health(&state).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_fields() {
        let body = service::build();
        assert_eq!(body.status, "ok");
        assert_eq!(body.service, "cngokzapi");
    }
}
