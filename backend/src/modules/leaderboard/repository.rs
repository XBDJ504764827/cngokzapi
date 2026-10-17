//! 排行榜数据源：读本站 `player_scores`（PROJECT §9.4）。
//! 本站汇总分 `points_local` 优先，缺失时回退上游口径 `points`。

use sqlx::{FromRow, PgPool};

use super::models::LeaderboardEntry;
use crate::modules::scoring;

/// 每次取榜条数（前端在返回集合内做搜索/筛选/排序）。
pub const BOARD_SIZE: i64 = 200;

#[derive(FromRow)]
struct Row {
    name: String,
    steam_id: String,
    avatar: Option<String>,
    maps_completed: i32,
    total_points: i64,
    records: i64,
    last_played: Option<String>,
    days_ago: Option<i64>,
}

const SQL: &str = r#"
SELECT p.name                                   AS name,
       p.steam_id::TEXT                         AS steam_id,
       p.avatar_url                             AS avatar,
       ps.maps_completed                        AS maps_completed,
       ps.points                                AS total_points,
       COALESCE(r.cnt, 0)                       AS records,
       to_char(r.last_played, 'YYYY-MM-DD')      AS last_played,
       CASE WHEN r.last_played IS NULL THEN NULL
            ELSE floor(EXTRACT(EPOCH FROM (now() - r.last_played)) / 86400)::BIGINT
       END                                      AS days_ago
FROM player_scores ps
JOIN players p ON p.steam_id = ps.player_steam_id
LEFT JOIN LATERAL (
    SELECT count(*) AS cnt, max(created_at) AS last_played
    FROM records
    WHERE player_steam_id = ps.player_steam_id AND mode = ps.mode
) r ON TRUE
WHERE ps.mode = $1
ORDER BY total_points DESC, ps.rank ASC NULLS LAST
LIMIT $2
"#;

/// 某模式榜单：按**积分**倒序，`rank` 为本站名次（PROJECT §9.4 独立计分）。
///
/// 积分取上游口径 `player_scores.points`：本站本地汇总 `points_local` 需在玩家成绩被
/// **完整**同步后才等价，当前是「已同步部分」的合计，故榜单不采用它（见 PROJECT §9.4）。
/// 上游全球排名（按 rating）保存在 `player_scores.rank`，供个人资料页展示。
pub async fn top(
    pool: &PgPool,
    mode: &str,
    limit: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(SQL)
        .bind(mode)
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| LeaderboardEntry {
            rank: Some((i + 1) as i32),
            player: r.name,
            steam_id64: r.steam_id,
            avatar: r.avatar.unwrap_or_default(),
            maps_completed: r.maps_completed,
            total_points: r.total_points,
            // 平均分口径由本站计分模块统一（PROJECT §9.4）
            avg_points: scoring::average_points(r.total_points, r.maps_completed.max(0) as u32),
            records: r.records,
            last_played: r.last_played,
            last_played_days_ago: r.days_ago,
        })
        .collect())
}
