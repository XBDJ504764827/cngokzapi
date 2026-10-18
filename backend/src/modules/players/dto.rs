use serde::Deserialize;

/// GET /api/v1/players/resolve?q=... —— 统一解析 SteamID / SteamID64 / 个人主页链接。
#[derive(Deserialize)]
pub struct ResolveQuery {
    pub q: Option<String>,
}
