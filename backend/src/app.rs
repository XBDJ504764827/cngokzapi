//! 路由和应用装配：`/api/v1/*` 各模块路由 + 前端静态托管（SPA 回退） + 中间件。

use std::path::Path;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::{errors::AppError, middleware, modules, state::AppState};

/// API 内未匹配到路由时返回统一 JSON 404（不能落到前端 index.html）。
async fn api_not_found() -> AppError {
    AppError::NotFound("接口不存在".to_string())
}

/// 装配全部路由。API 优先匹配；静态目录作 fallback，未知路径回退 `index.html`
/// 以支持前端 history 路由（如 `/maps/11`、`/profile/<steamid64>`）。
/// `web_dir` 缺失（如前端尚未构建）时只告警，API 照常可用。
pub fn create_app(state: AppState) -> Router {
    let api = Router::new()
        .merge(modules::health::routes::routes())
        .merge(modules::leaderboard::routes::routes())
        .merge(modules::records::routes::routes())
        .merge(modules::bans::routes::routes())
        .merge(modules::maps::routes::routes())
        .merge(modules::servers::routes::routes())
        .merge(modules::players::routes::routes())
        .merge(modules::stats::routes::routes())
        .merge(modules::ingest::routes::routes())
        .fallback(api_not_found)
        .with_state(state.clone());

    // GlobalAPI 兼容层（PROJECT §9.6）：官方客户端的 POST /records
    let compat = Router::new()
        .merge(modules::ingest::routes::compat_routes())
        .fallback(api_not_found)
        .with_state(state.clone());

    let web_dir = state.config.web_dir.clone();
    let index_file = Path::new(&web_dir).join("index.html");
    if !index_file.exists() {
        tracing::warn!(
            web_dir = %web_dir,
            "前端构建产物缺失（先执行 frontend: npm run build），/ 暂回 404，API 不受影响"
        );
    }
    let static_service = ServeDir::new(&web_dir)
        .append_index_html_on_directories(true)
        // SPA 回退：静态文件不存在时返回 index.html（状态码保持 200，交给前端路由）
        .fallback(ServeFile::new(index_file));

    Router::new()
        .nest("/api/v1", api)
        .nest("/api/v2.0", compat)
        .fallback_service(static_service)
        .layer(middleware::cors::dev_cors())
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::*;

    /// 默认路由可构建；SPA 回退不吞掉 API 的 404。
    #[tokio::test]
    async fn api_404_is_json_not_index() {
        let config = crate::config::AppConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            web_dir: "/nonexistent-web-dir".to_string(),
            database_url: "postgres://user:pass@127.0.0.1:1/db".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            steam_api_key: String::new(),
        };
        let app = create_app(AppState::new(config));
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/definitely-not-a-route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let ct = res
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        assert!(ct.contains("application/json"), "content-type = {ct}");
    }
}
