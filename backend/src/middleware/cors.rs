//! CORS：开发期放行 Vite 开发服（5173）跨域调 `/api`；生产为同源部署，不受影响。

use tower_http::cors::{Any, CorsLayer};

/// 开发期宽松 CORS。后端鉴权上线后收紧为显式 origin 白名单。
pub fn dev_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}
