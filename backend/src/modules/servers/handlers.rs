use axum::{extract::State, Json};

use super::{models::ServerEntry, service};
use crate::{errors::AppError, state::AppState};

/// GET /api/v1/servers —— 在线服务器列表（公开）。
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<ServerEntry>>, AppError> {
    Ok(Json(service::list(&state).await?))
}
