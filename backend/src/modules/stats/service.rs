//! 编排层：首页统计。

use super::{models::HomeStats, repository};
use crate::{errors::AppError, state::AppState};

pub async fn home(state: &AppState) -> Result<HomeStats, AppError> {
    Ok(repository::home(&state.db).await?)
}
