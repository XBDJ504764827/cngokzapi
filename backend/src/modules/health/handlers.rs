use axum::Json;

use super::{models::HealthResponse, service};

/// GET /api/v1/health —— 无需鉴权。
pub async fn health() -> Json<HealthResponse> {
    Json(service::build())
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
