//! 排行榜数据源：读本站 `player_scores`（PROJECT §9.4）。
//! 榜单按上游总积分排序；搜索 / 活跃度 / 排序 / 分页全部在 SQL 完成。

use sqlx::{FromRow, PgPool};

use super::models::LeaderboardEntry;
use crate::modules::scoring;

/// 默认 / 最大每页条数。
pub const DEFAULT_PAGE_SIZE: i64 = 50;
pub const MAX_PAGE_SIZE: i64 = 100;

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

/// 活跃度过滤（基于本地 `records.created_at`；`inactive` 含本地无成绩的玩家）。
const ACTIVITY_FILTER: &str = r#"(
    $3 = 'all'
    OR ($3 = '7' AND EXISTS (
        SELECT 1 FROM records ra
        WHERE ra.player_steam_id = ps.player_steam_id AND ra.mode = ps.mode
          AND ra.is_valid AND ra.created_at > now() - interval '7 days'
    ))
    OR ($3 = '30' AND EXISTS (
        SELECT 1 FROM records ra
        WHERE ra.player_steam_id = ps.player_steam_id AND ra.mode = ps.mode
          AND ra.is_valid AND ra.created_at > now() - interval '30 days'
    ))
    OR ($3 = 'inactive' AND NOT EXISTS (
        SELECT 1 FROM records ra
        WHERE ra.player_steam_id = ps.player_steam_id AND ra.mode = ps.mode
          AND ra.is_valid AND ra.created_at > now() - interval '30 days'
    ))
)"#;

/// 某模式分页榜单，返回 `(条目, 总数)`。
///
/// 排序口径：积分取上游 `player_scores.points`（本站 `points_local` 需完整同步后才等价，
/// 见 PROJECT §9.4）；`avg` 为总积分 / 完成地图数。
pub async fn page(
    pool: &PgPool,
    mode: &str,
    query: &str,
    activity: &str,
    sort: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<LeaderboardEntry>, i64), sqlx::Error> {
    let order_by = match sort {
        "maps" => "ps.maps_completed DESC, ps.points DESC, p.steam_id",
        "avg" => {
            "(ps.points::DOUBLE PRECISION / GREATEST(ps.maps_completed, 1)) DESC, \
             ps.points DESC, p.steam_id"
        }
        "rating" => "ps.rating DESC NULLS LAST, ps.points DESC, p.steam_id",
        _ => "ps.points DESC, ps.rank ASC NULLS LAST, p.steam_id",
    };

    let sql = format!(
        r#"
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
            WHERE player_steam_id = ps.player_steam_id AND mode = ps.mode AND is_valid
        ) r ON TRUE
        WHERE ps.mode = $1 AND ps.points > 0
          AND ($2 = ''
               OR p.name ILIKE '%' || $2 || '%'
               OR p.steam_id::TEXT LIKE '%' || $2 || '%')
          AND {ACTIVITY_FILTER}
        ORDER BY {order_by}
        LIMIT $4 OFFSET $5
        "#
    );

    let rows: Vec<Row> = sqlx::query_as(&sql)
        .bind(mode)
        .bind(query)
        .bind(activity)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let count_sql = format!(
        r#"
        SELECT count(*)::BIGINT
        FROM player_scores ps
        JOIN players p ON p.steam_id = ps.player_steam_id
        WHERE ps.mode = $1 AND ps.points > 0
          AND ($2 = ''
               OR p.name ILIKE '%' || $2 || '%'
               OR p.steam_id::TEXT LIKE '%' || $2 || '%')
          AND {ACTIVITY_FILTER}
        "#
    );
    let (total,): (i64,) = sqlx::query_as(&count_sql)
        .bind(mode)
        .bind(query)
        .bind(activity)
        .fetch_one(pool)
        .await?;

    let data = rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| LeaderboardEntry {
            rank: Some((offset + i as i64 + 1) as i32),
            player: r.name,
            steam_id64: r.steam_id,
            avatar: r.avatar.unwrap_or_default(),
            maps_completed: r.maps_completed,
            total_points: r.total_points,
            avg_points: scoring::average_points(r.total_points, r.maps_completed.max(0) as u32),
            records: r.records,
            last_played: r.last_played,
            last_played_days_ago: r.days_ago,
        })
        .collect();

    Ok((data, total))
}
