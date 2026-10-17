//! 编排层：取三模式最新记录。数据由 `modules/sync` 从外部源同步进 PG。

use super::{models::RecordEntry, repository};
use crate::{errors::AppError, modules::mode, state::AppState};

/// 每个模式返回的最新记录条数（前端三列并排各取 20）。
pub const PER_MODE: i64 = 20;

pub async fn list(state: &AppState) -> Result<Vec<RecordEntry>, AppError> {
    let mut out = Vec::with_capacity((PER_MODE as usize) * mode::MODES.len());
    for m in mode::MODES {
        out.extend(repository::latest_by_mode(&state.db, m, PER_MODE).await?);
    }
    Ok(out)
}
