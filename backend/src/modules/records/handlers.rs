use axum::Json;

use super::{models::RecordEntry, service};

/// GET /api/v1/records —— 公开接口，无需鉴权。
pub async fn list() -> Json<Vec<RecordEntry>> {
    Json(service::list())
}
