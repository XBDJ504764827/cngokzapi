use axum::{extract::State, Json};

use super::{models::RecordEntry, service};
use crate::{errors::AppError, state::AppState};

/// GET /api/v1/records —— 公开接口，返回三模式各最新 20 条。
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<RecordEntry>>, AppError> {
    Ok(Json(service::list(&state).await?))
}
