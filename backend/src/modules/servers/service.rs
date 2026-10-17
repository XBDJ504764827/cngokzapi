//! 编排层：在线服务器列表。

use super::{models::ServerEntry, repository};
use crate::{errors::AppError, state::AppState};

pub async fn list(state: &AppState) -> Result<Vec<ServerEntry>, AppError> {
    Ok(repository::list_online(&state.db).await?)
}
