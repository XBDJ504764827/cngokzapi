use serde::Serialize;

/// 健康检查响应体，字段保持稳定，监控与部署脚本可直接依赖。
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

/// 就绪检查响应体：如实反映依赖状态，不可用只降级不报错。
#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    pub postgres: bool,
    pub redis: bool,
}
