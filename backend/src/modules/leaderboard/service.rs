//! 编排层：模式校验 + 取榜。榜单数据由 `modules/sync` 定时/按需同步进 PG。

use super::{models::LeaderboardEntry, repository};
use crate::{errors::AppError, modules::mode, state::AppState};

/// 取某模式榜单；本地无数据时返回空表（同步由后台 worker 负责）。
pub async fn board(state: &AppState, m: Option<&str>) -> Result<Vec<LeaderboardEntry>, AppError> {
    let m = mode::validate(m)?;
    Ok(repository::top(&state.db, m, repository::BOARD_SIZE).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_mode() {
        assert!(mode::validate(Some("XXX")).is_err());
        assert_eq!(mode::validate(None).unwrap(), "KZT");
        assert_eq!(mode::validate(Some("VNL")).unwrap(), "VNL");
    }
}
