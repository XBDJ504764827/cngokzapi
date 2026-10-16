//! 路由和应用装配：`/api/v1/*` 各模块路由 + 前端静态托管 + 中间件。

use axum::Router;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{middleware, modules, state::AppState};

/// 装配全部路由。API 优先匹配，静态目录作 fallback；
/// `web_dir` 缺失（如前端尚未构建）时只告警，API 照常可用。
pub fn create_app(state: AppState) -> Router {
    let api = Router::new()
        .merge(modules::health::routes::routes())
        .merge(modules::leaderboard::routes::routes())
        .merge(modules::records::routes::routes())
        .merge(modules::bans::routes::routes())
        .with_state(state.clone());

    let web_dir = state.config.web_dir.clone();
    if !std::path::Path::new(&web_dir).exists() {
        tracing::warn!(
            web_dir = %web_dir,
            "前端构建产物缺失（先执行 frontend: npm run build），/ 暂回 404，API 不受影响"
        );
    }
    let static_service = ServeDir::new(web_dir).append_index_html_on_directories(true);

    Router::new()
        .nest("/api/v1", api)
        .fallback_service(static_service)
        .layer(middleware::cors::dev_cors())
        .layer(TraceLayer::new_for_http())
}
