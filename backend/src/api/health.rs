use axum::Json;
use serde::Serialize;

/// 健康检查响应体，保持字段稳定，监控与部署脚本可直接依赖。
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

/// GET /api/v1/health —— 无需鉴权，进程存活即返回 ok。
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "cngokzapi",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_body_fields() {
        // 不经过 HTTP 栈，直接验证 handler 返回的 JSON 字段，避免端口占用。
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let body = rt.block_on(health()).0;
        assert_eq!(body.status, "ok");
        assert_eq!(body.service, "cngokzapi");
        assert_eq!(body.version, env!("CARGO_PKG_VERSION"));
    }
}
