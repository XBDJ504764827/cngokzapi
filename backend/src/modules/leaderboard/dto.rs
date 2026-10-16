use serde::Deserialize;

/// GET /api/v1/leaderboard?mode=KZT —— 缺省 KZT。
#[derive(Deserialize)]
pub struct LeaderboardQuery {
    pub mode: Option<String>,
}
