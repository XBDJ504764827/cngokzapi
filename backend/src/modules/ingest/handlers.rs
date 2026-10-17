//! 上报接收端 handler：薄层，只做头部萃取、批量上限校验与错误映射。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use super::{
    dto::{CompatRecord, IngestItem},
    models::{CompatResponse, IngestResponse},
    service,
};
use crate::{errors::AppError, state::AppState};

/// 从 `X-ApiKey` 取服务器组 key。
fn api_key(headers: &HeaderMap) -> Option<&str> {
    headers.get("x-apikey").and_then(|v| v.to_str().ok())
}

/// POST /api/v1/ingest/records —— 自建 `cngokzapi` 插件的批量上报。
pub async fn ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(items): Json<Vec<IngestItem>>,
) -> Result<Json<IngestResponse>, AppError> {
    let auth = service::authenticate(&state, api_key(&headers)).await?;
    if items.is_empty() {
        return Err(AppError::BadRequest("records 不能为空".to_string()));
    }
    if items.len() > service::MAX_BATCH {
        return Err(AppError::BadRequest(format!(
            "单批最多 {} 条，收到 {}",
            service::MAX_BATCH,
            items.len()
        )));
    }
    let results = service::ingest_batch(&state, &auth, &items).await?;
    tracing::info!(
        group = %auth.group_name,
        group_id = auth.group_id,
        received = items.len(),
        accepted = results.iter().filter(|r| r.status == "accepted").count(),
        duplicate = results.iter().filter(|r| r.status == "duplicate").count(),
        rejected = results.iter().filter(|r| r.status == "rejected").count(),
        "上报接收"
    );
    Ok(Json(IngestResponse { results }))
}

/// POST /api/v2.0/records —— GlobalAPI 兼容层（字段与官方客户端一致）。
pub async fn compat_record(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(item): Json<CompatRecord>,
) -> Result<(StatusCode, Json<CompatResponse>), AppError> {
    let auth = service::authenticate(&state, api_key(&headers)).await?;
    let resp = service::ingest_compat(&state, &auth, &item).await?;
    tracing::info!(
        group = %auth.group_name,
        map_id = item.map_id,
        place = resp.place,
        "GlobalAPI 兼容上报"
    );
    Ok((StatusCode::CREATED, Json(resp)))
}
