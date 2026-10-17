use serde::Deserialize;

/// GET /api/v1/maps?query=&tier=&mode= —— 地图列表筛选。
/// 列表接口一次性返回全部地图（约 1100 条），前端本地筛选；
/// 参数保留给后续服务端筛选使用。
#[derive(Deserialize)]
pub struct MapsQuery {
    pub query: Option<String>,
    pub tier: Option<i16>,
    pub mode: Option<String>,
}

/// GET /api/v1/maps/{id}?mode=KZT —— 详情按模式取榜。
#[derive(Deserialize)]
pub struct MapDetailQuery {
    pub mode: Option<String>,
}
