//! 编排层：模式校验 + 取数。后续在此加 Redis 榜单缓存（TTL 60s）与 PG 回源。

use super::{models::LeaderboardEntry, repository};
use crate::{errors::AppError, modules::mock};

/// 取整榜。`mode` 缺省 KZT，非三模式直接 400。
pub fn board(mode: Option<&str>) -> Result<Vec<LeaderboardEntry>, AppError> {
    let mode = mode.unwrap_or("KZT");
    if !mock::MODES.contains(&mode) {
        return Err(AppError::BadRequest(format!(
            "mode 非法，仅支持 {}",
            mock::MODES.join("/")
        )));
    }
    // TODO: 先查 Redis（键 leaderboard:{mode}），未命中查 PG 并回填。
    Ok(repository::mock_board(mode))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::leaderboard::repository::BOARD_SIZE;

    #[test]
    fn rejects_bad_mode() {
        assert!(board(Some("XXX")).is_err());
        assert_eq!(board(None).unwrap().len(), BOARD_SIZE);
    }
}
