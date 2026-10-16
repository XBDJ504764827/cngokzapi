use super::models::HealthResponse;

/// 存活探针：进程存活即 ok，不依赖 DB/Redis。
pub fn build() -> HealthResponse {
    HealthResponse {
        status: "ok",
        service: "cngokzapi",
        version: env!("CARGO_PKG_VERSION"),
    }
}
