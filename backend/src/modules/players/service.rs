//! 编排层：个人资料按需懒加载（PROJECT §9.5）。
//! 首次访问（本地无该玩家数据）时入队后台同步，立即返回已有数据 + `syncing = true`。

use super::{models::PlayerProfile, repository};
use crate::{errors::AppError, modules::sync, state::AppState};

/// 最近成绩返回条数。
pub const RECENT_LIMIT: i64 = 50;

/// SteamID64：17 位纯数字。
pub fn parse_steam_id(raw: &str) -> Result<i64, AppError> {
    if raw.len() != 17 || !raw.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "steam_id64 应为 17 位数字".to_string(),
        ));
    }
    raw.parse::<i64>()
        .map_err(|_| AppError::BadRequest("steam_id64 非法".to_string()))
}

pub async fn profile(state: &AppState, raw: &str) -> Result<PlayerProfile, AppError> {
    let steam_id = parse_steam_id(raw)?;
    let scores = repository::scores(&state.db, steam_id).await?;
    let player = repository::find_player(&state.db, steam_id).await?;

    // 是否已成功同步过该玩家（按 sync_jobs 的 done 状态判断）：
    // 榜单导入只带来排名/积分，成绩与最近游玩需要玩家级同步，故首次访问仍会入队。
    let synced = sync::queue::is_done(&state.db, sync::KIND_PLAYER, raw).await?;
    if !synced {
        sync::worker::enqueue_player(state, raw).await;
    }

    let recent_records = if player.is_some() {
        repository::recent_records(&state.db, steam_id, RECENT_LIMIT).await?
    } else {
        Vec::new()
    };
    let (name, avatar, region) = match player {
        Some(p) => (p.name, p.avatar.unwrap_or_default(), p.region),
        None => (String::new(), String::new(), None),
    };

    Ok(PlayerProfile {
        steam_id64: raw.to_string(),
        name,
        avatar,
        region,
        syncing: !synced,
        scores,
        recent_records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_steam_id() {
        assert!(parse_steam_id("76561198297415967").is_ok());
        assert!(parse_steam_id("123").is_err());
        assert!(parse_steam_id("7656119829741596a").is_err());
        assert!(parse_steam_id("").is_err());
    }
}
