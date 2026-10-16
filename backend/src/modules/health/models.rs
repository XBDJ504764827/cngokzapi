use serde::Serialize;

/// 健康检查响应体，字段保持稳定，监控与部署脚本可直接依赖。
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}
