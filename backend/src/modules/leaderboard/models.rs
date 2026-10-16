//! 排行榜条目。JSON 一律 snake_case，与前端 `features/leaderboard/types.ts` 一一对应。

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player: String,
    pub steam_id64: String,
    pub avatar: String,
    pub maps_completed: u32,
    pub total_points: u64,
    pub avg_points: f32,
    pub records: u32,
    pub last_played: String,
    pub last_played_days_ago: u32,
}
