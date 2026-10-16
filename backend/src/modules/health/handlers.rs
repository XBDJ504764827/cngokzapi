use axum::{extract::State, Json};

use super::{
    models::{HealthResponse, ReadyResponse},
    service,
};
use crate::state::AppState;

/// GET /api/v1/health —— 无需鉴权。
pub async fn health() -> Json<HealthResponse> {
    Json(service::build())
}

/// GET /api/v1/health/ready —— 无需鉴权，返回 PG/Redis 连通性。
pub async fn ready(State(state): State<AppState>) -> Json<ReadyResponse> {
    Json(service::readiness(&state).await)
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
