//! 编排层：个人资料按需懒加载（PROJECT §9.5）。
//! 首次访问（本地无该玩家数据）时入队后台同步，立即返回已有数据 + `syncing = true`。

use super::{
    dto::PlayerMapsQuery,
    models::{PlayerMaps, PlayerProfile},
    repository,
};
use crate::{
    errors::AppError,
    modules::{mode, sync},
    sources::{steam::Steam, SourceError},
    state::AppState,
};

/// 每个模式返回的最近有分成绩条数。
pub const RECENT_PER_MODE: i64 = 20;

/// 「地图进度」每页条数（与地图列表一致的 24）。
pub const MAPS_PER_PAGE: i64 = 24;
const MAPS_PER_PAGE_MAX: i64 = 100;

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
    if synced {
        // 7 天 TTL：到期后后台重拉 PB / 最近成绩，修正上游硬删除与改分
        sync::queue::enqueue_if_stale(&state.db, sync::KIND_PLAYER, raw, sync::DATA_TTL_SECS)
            .await?;
    } else {
        sync::worker::enqueue_player(state, raw).await;
    }

    let recent_records = if player.is_some() {
        // 按模式分别取，避免某一模式占满列表（前端也按模式分组展示）
        let mut out = Vec::with_capacity((RECENT_PER_MODE as usize) * mode::MODES.len());
        for m in mode::MODES {
            out.extend(repository::recent_records(&state.db, steam_id, m, RECENT_PER_MODE).await?);
        }
        out
    } else {
        Vec::new()
    };
    let (name, avatar, country, region) = match player {
        Some(p) => (p.name, p.avatar.unwrap_or_default(), p.country, p.region),
        None => (String::new(), String::new(), None, None),
    };

    Ok(PlayerProfile {
        steam_id64: raw.to_string(),
        name,
        avatar,
        country,
        region,
        syncing: !synced,
        scores,
        recent_records,
    })
}

// ------------------------------ 地图进度 ------------------------------

/// 玩家「地图进度」：某模式下的已完成 / 未完成地图，分页 + 名称搜索。
pub async fn player_maps(
    state: &AppState,
    raw: &str,
    query: &PlayerMapsQuery,
) -> Result<PlayerMaps, AppError> {
    let steam_id = parse_steam_id(raw)?;
    let mode = mode::validate(query.mode.as_deref())?;
    let status = match query.status.as_deref() {
        Some("uncompleted") => "uncompleted",
        _ => "completed",
    };
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query
        .per_page
        .unwrap_or(MAPS_PER_PAGE)
        .clamp(1, MAPS_PER_PAGE_MAX);
    let q = query.q.as_deref().unwrap_or("").trim().to_string();
    let offset = (page - 1) * per_page;

    let completed_total = repository::count_completed_maps(&state.db, steam_id, mode).await?;
    let maps_total = repository::count_maps(&state.db).await?;
    let (data, total) = if status == "completed" {
        (
            repository::completed_maps(&state.db, steam_id, mode, &q, per_page, offset).await?,
            repository::count_completed_maps_filtered(&state.db, steam_id, mode, &q).await?,
        )
    } else {
        (
            repository::uncompleted_maps(&state.db, steam_id, mode, &q, per_page, offset).await?,
            repository::count_uncompleted_maps(&state.db, steam_id, mode, &q).await?,
        )
    };

    Ok(PlayerMaps {
        mode: mode.to_string(),
        status: status.to_string(),
        page,
        per_page,
        total,
        completed_total,
        maps_total,
        data,
    })
}

// ------------------------------ SteamID 解析 ------------------------------

const STEAM64_BASE: i64 = 76_561_197_960_265_728;

/// 个人主页链接解析结果。
enum ProfileRef {
    Id64(String),
    Vanity(String),
}

fn is_steam64(s: &str) -> bool {
    s.len() == 17 && s.chars().all(|c| c.is_ascii_digit())
}

/// `STEAM_X:Y:Z` → SteamID64（`base + Z*2 + Y`）。
fn parse_steam2(s: &str) -> Option<String> {
    let rest = s
        .strip_prefix("STEAM_")
        .or_else(|| s.strip_prefix("steam_"))?;
    let mut parts = rest.split(':');
    let _universe = parts.next()?;
    let y: i64 = parts.next()?.parse().ok()?;
    let z: i64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(0..=1).contains(&y) || z < 0 {
        return None;
    }
    Some((STEAM64_BASE + z * 2 + y).to_string())
}

/// `[U:1:W]` / `U:1:W` → SteamID64（`base + W`）。
fn parse_steam3(s: &str) -> Option<String> {
    let t = s.trim().trim_start_matches('[').trim_end_matches(']');
    let account = t.strip_prefix("U:1:").or_else(|| t.strip_prefix("u:1:"))?;
    let w: i64 = account.parse().ok()?;
    if w < 0 {
        return None;
    }
    Some((STEAM64_BASE + w).to_string())
}

/// 纯数字账号 id（1-10 位，SteamID3 数字形式）→ SteamID64。
fn parse_account_id(s: &str) -> Option<String> {
    if s.is_empty() || s.len() > 10 || !s.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let w: i64 = s.parse().ok()?;
    Some((STEAM64_BASE + w).to_string())
}

/// Steam 个人主页链接：`/profiles/<SteamID64>` 或 `/id/<自定义名>`。
fn parse_profile_url(s: &str) -> Option<ProfileRef> {
    let candidate = if s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else {
        format!("https://{s}")
    };
    let url = reqwest::Url::parse(&candidate).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    if host != "steamcommunity.com" && !host.ends_with(".steamcommunity.com") {
        return None;
    }
    let segments: Vec<&str> = url.path_segments()?.filter(|p| !p.is_empty()).collect();
    match segments.as_slice() {
        ["profiles", id] if is_steam64(id) => Some(ProfileRef::Id64((*id).to_string())),
        ["id", vanity] if !vanity.is_empty() && vanity.len() <= 64 => {
            Some(ProfileRef::Vanity((*vanity).to_string()))
        }
        _ => None,
    }
}

fn source_error(e: SourceError) -> AppError {
    match e {
        SourceError::Rejected(_, m) | SourceError::Unavailable(m) | SourceError::Decode(m) => {
            AppError::ServiceUnavailable(format!("Steam 查询失败：{m}"))
        }
    }
}

/// 将用户输入统一解析为 SteamID64（个人主页 `/id/xxx` 通过 Steam Web API 解析）。
pub async fn resolve(state: &AppState, raw: &str) -> Result<String, AppError> {
    let q = raw.trim();
    if q.is_empty() {
        return Err(AppError::BadRequest(
            "请输入 SteamID / SteamID64 / Steam 个人主页链接".to_string(),
        ));
    }
    if is_steam64(q) {
        return Ok(q.to_string());
    }
    if let Some(id) = parse_steam2(q) {
        return Ok(id);
    }
    if let Some(id) = parse_steam3(q) {
        return Ok(id);
    }
    if let Some(id) = parse_account_id(q) {
        return Ok(id);
    }
    match parse_profile_url(q) {
        Some(ProfileRef::Id64(id)) => Ok(id),
        Some(ProfileRef::Vanity(vanity)) => {
            let key = state.config.steam_api_key.trim();
            if key.is_empty() {
                return Err(AppError::ServiceUnavailable(
                    "未配置 STEAM_API_KEY，无法解析自定义主页链接".to_string(),
                ));
            }
            let id = Steam(state.source.as_ref(), key)
                .resolve_vanity(&vanity)
                .await
                .map_err(source_error)?;
            id.ok_or_else(|| AppError::NotFound("未找到该 Steam 自定义主页".to_string()))
        }
        None => Err(AppError::BadRequest(
            "无法识别的输入；支持 SteamID（STEAM_x:y:z）、SteamID3（[U:1:w]）、SteamID64、\
             Steam 个人主页链接（/profiles/… 或 /id/…）"
                .to_string(),
        )),
    }
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

    #[test]
    fn resolves_steam_id_formats() {
        // Flonnych：STEAM_1:1:168575119 = 76561198297415967
        assert_eq!(
            parse_steam2("STEAM_1:1:168575119").as_deref(),
            Some("76561198297415967")
        );
        assert_eq!(
            parse_steam3("[U:1:337150239]").as_deref(),
            Some("76561198297415967")
        );
        assert_eq!(
            parse_account_id("337150239").as_deref(),
            Some("76561198297415967")
        );
        assert!(matches!(
            parse_profile_url("https://steamcommunity.com/profiles/76561198297415967"),
            Some(ProfileRef::Id64(_))
        ));
        assert!(matches!(
            parse_profile_url("steamcommunity.com/id/flonnych/"),
            Some(ProfileRef::Vanity(_))
        ));
        assert!(parse_steam2("not-a-steamid").is_none());
    }
}
