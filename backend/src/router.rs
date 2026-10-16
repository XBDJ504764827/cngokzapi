use axum::{routing::get, Router};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::api::health;

/// 装配全部路由：`/` 与静态资源由 `web/` 目录托管（与 ~/cngokz-web 一模一样），
/// `/api/v1/*` 为版本化 JSON API（优先匹配，不会被静态 fallback 吞掉）。
///
/// `web_dir` 为 workspace 根下的 `web/` 目录（可用 WEB_DIR 覆盖），
/// `append_index_html_on_directories(true)` 让 `/` 自动返回 `index.html`。
pub fn create_router_with_web_dir(web_dir: &str) -> Router {
    let static_service = ServeDir::new(web_dir).append_index_html_on_directories(true);

    Router::new()
        .nest("/api/v1", api_v1_router())
        .fallback_service(static_service)
        .layer(TraceLayer::new_for_http())
}

fn api_v1_router() -> Router {
    Router::new().route("/health", get(health::health))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_router_builds() {
        // 装配不断言文件存在，只验证路由构建不 panic。
        let _ = create_router_with_web_dir("web");
    }
}
