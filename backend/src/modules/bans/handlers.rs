use axum::{
    extract::{Query, State},
    Json,
};

use super::{dto::BansQuery, models::BanEntry, service};
use crate::{errors::AppError, modules::page::Page, state::AppState};

/// GET /api/v1/bans —— 公开接口，返回分页信封。
pub async fn page(
    State(_state): State<AppState>,
    Query(q): Query<BansQuery>,
) -> Result<Json<Page<BanEntry>>, AppError> {
    Ok(Json(service::page(q)?))
}
