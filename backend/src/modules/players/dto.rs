use serde::Deserialize;

/// GET /api/v1/players/resolve?q=... —— 统一解析 SteamID / SteamID64 / 个人主页链接。
#[derive(Deserialize)]
pub struct ResolveQuery {
    pub q: Option<String>,
}

/// GET /api/v1/players/{steam_id64}/maps —— 玩家地图进度（已完成 / 未完成）。
#[derive(Deserialize)]
pub struct PlayerMapsQuery {
    pub mode: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub q: Option<String>,
}
