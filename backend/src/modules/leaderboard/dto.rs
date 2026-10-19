use serde::Deserialize;

/// GET /api/v1/leaderboard?mode=&page=&per_page=&query=&sort=&activity=
/// - `sort`：`points`（默认）/ `maps` / `avg` / `rating`
/// - `activity`：`all`（默认）/ `7` / `30` / `inactive`
#[derive(Deserialize)]
pub struct LeaderboardQuery {
    pub mode: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub query: Option<String>,
    pub sort: Option<String>,
    pub activity: Option<String>,
}
