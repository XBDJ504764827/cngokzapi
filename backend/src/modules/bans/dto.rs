use serde::Deserialize;

/// GET /api/v1/bans?query=&page=1&per_page=20 —— 名称模糊/SteamID64 子串搜索 + 分页。
#[derive(Deserialize)]
pub struct BansQuery {
    pub query: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}
