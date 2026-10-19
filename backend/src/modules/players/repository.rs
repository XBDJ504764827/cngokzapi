//! 个人资料数据源：读本站 `players` / `player_scores` / `records`。

use sqlx::{FromRow, PgPool};

use super::models::{PlayerMapEntry, PlayerRecordEntry, PlayerScoreEntry};
use crate::modules::format::format_duration_ms;

#[derive(FromRow)]
pub struct PlayerRow {
    pub name: String,
    pub avatar: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
}

/// 玩家基础档案。
pub async fn find_player(pool: &PgPool, steam_id: i64) -> Result<Option<PlayerRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT name, avatar_url AS avatar, country, region FROM players WHERE steam_id = $1",
    )
    .bind(steam_id)
    .fetch_optional(pool)
    .await
}

#[derive(FromRow)]
struct ScoreRow {
    mode: String,
    rank: Option<i32>,
    points: i64,
    points_local: Option<i64>,
    rating: Option<f64>,
    maps_completed: i32,
    wrs_nub: i32,
    wrs_pro: i32,
    records: i64,
}

/// 三模式档案：`points` 为上游全球积分，`points_local` 为本站已同步成绩的合计
/// （仅在成绩被完整同步后才与上游等价，见 PROJECT §9.4）。
pub async fn scores(pool: &PgPool, steam_id: i64) -> Result<Vec<PlayerScoreEntry>, sqlx::Error> {
    let rows: Vec<ScoreRow> = sqlx::query_as(
        r#"SELECT ps.mode,
                  ps.rank,
                  ps.points                            AS points,
                  ps.points_local                      AS points_local,
                  ps.rating,
                  ps.maps_completed,
                  ps.wrs_nub,
                  ps.wrs_pro,
                  COALESCE(r.cnt, 0)                   AS records
           FROM player_scores ps
           LEFT JOIN LATERAL (
               SELECT count(*) AS cnt FROM records
               WHERE player_steam_id = ps.player_steam_id AND mode = ps.mode
                 AND points > 0
           ) r ON TRUE
           WHERE ps.player_steam_id = $1
           ORDER BY ps.mode"#,
    )
    .bind(steam_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PlayerScoreEntry {
            mode: r.mode,
            rank: r.rank,
            points: r.points,
            points_local: r.points_local,
            rating: r.rating,
            maps_completed: r.maps_completed,
            wrs_nub: r.wrs_nub,
            wrs_pro: r.wrs_pro,
            records: r.records,
        })
        .collect())
}

#[derive(FromRow)]
struct RecordRow {
    map_id: i64,
    map: String,
    mode: String,
    stage: i32,
    time_ms: i32,
    points: i32,
    date: String,
}

/// 某玩家某模式的最近有分成绩（按上传时间倒序）。
pub async fn recent_records(
    pool: &PgPool,
    steam_id: i64,
    mode: &str,
    limit: i64,
) -> Result<Vec<PlayerRecordEntry>, sqlx::Error> {
    let rows: Vec<RecordRow> = sqlx::query_as(
        r#"SELECT m.id                                 AS map_id,
                  m.name                               AS map,
                  r.mode                               AS mode,
                  r.stage                              AS stage,
                  r.time_ms                            AS time_ms,
                  r.points                             AS points,
                  to_char(r.created_at, 'YYYY-MM-DD')   AS date
           FROM records r
           JOIN maps m ON m.id = r.map_id
           WHERE r.player_steam_id = $1 AND r.mode = $2 AND r.points > 0
           ORDER BY r.created_at DESC
           LIMIT $3"#,
    )
    .bind(steam_id)
    .bind(mode)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PlayerRecordEntry {
            map_id: r.map_id,
            map: r.map,
            mode: r.mode,
            stage: r.stage,
            time: format_duration_ms(r.time_ms),
            points: i64::from(r.points),
            date: r.date,
        })
        .collect())
}

// ------------------------------ 地图进度（已完成 / 未完成） ------------------------------

/// 按模式取难度列的 SQL 片段（`$2` 为模式）。
const TIER_OF_MODE: &str = "CASE $2::text \
     WHEN 'KZT' THEN m.tier_kzt WHEN 'SKZ' THEN m.tier_skz ELSE m.tier_vnl END";

#[derive(FromRow)]
struct PlayerMapRow {
    map_id: i64,
    name: String,
    tier: i16,
    time_ms: Option<i32>,
    points: Option<i64>,
}

impl PlayerMapRow {
    fn into_entry(self, completed: bool) -> PlayerMapEntry {
        PlayerMapEntry {
            map_id: self.map_id,
            name: self.name,
            tier: self.tier,
            best_time: self.time_ms.map(format_duration_ms),
            points: self.points,
            completed,
        }
    }
}

/// 已完成地图（按名称升序），带主关最佳耗时与分数。
pub async fn completed_maps(
    pool: &PgPool,
    steam_id: i64,
    mode: &str,
    query: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<PlayerMapEntry>, sqlx::Error> {
    let rows: Vec<PlayerMapRow> = sqlx::query_as(&format!(
        r#"SELECT m.id AS map_id, m.name, {TIER_OF_MODE} AS tier,
                  best.time_ms AS time_ms, best.points::BIGINT AS points
           FROM maps m
           JOIN LATERAL (
                SELECT r.time_ms, r.points
                FROM records r
                WHERE r.map_id = m.id AND r.player_steam_id = $1 AND r.mode = $2
                  AND r.stage = 0 AND r.is_valid
                ORDER BY r.time_ms ASC
                LIMIT 1
           ) best ON TRUE
           WHERE ($3 = '' OR m.name ILIKE '%' || $3 || '%')
           ORDER BY m.name ASC
           LIMIT $4 OFFSET $5"#
    ))
    .bind(steam_id)
    .bind(mode)
    .bind(query)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into_entry(true)).collect())
}

/// 未完成地图（按名称升序）。
pub async fn uncompleted_maps(
    pool: &PgPool,
    steam_id: i64,
    mode: &str,
    query: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<PlayerMapEntry>, sqlx::Error> {
    let rows: Vec<PlayerMapRow> = sqlx::query_as(&format!(
        r#"SELECT m.id AS map_id, m.name, {TIER_OF_MODE} AS tier,
                  NULL::int AS time_ms, NULL::bigint AS points
           FROM maps m
           WHERE NOT EXISTS (
                   SELECT 1 FROM records r
                   WHERE r.map_id = m.id AND r.player_steam_id = $1 AND r.mode = $2
                     AND r.stage = 0 AND r.is_valid
                 )
             AND ($3 = '' OR m.name ILIKE '%' || $3 || '%')
           ORDER BY m.name ASC
           LIMIT $4 OFFSET $5"#
    ))
    .bind(steam_id)
    .bind(mode)
    .bind(query)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into_entry(false)).collect())
}

/// 已完成地图数（不受名称搜索影响）。
pub async fn count_completed_maps(
    pool: &PgPool,
    steam_id: i64,
    mode: &str,
) -> Result<i64, sqlx::Error> {
    let (n,): (i64,) = sqlx::query_as(
        r#"SELECT count(*)::BIGINT FROM maps m
           WHERE EXISTS (
               SELECT 1 FROM records r
               WHERE r.map_id = m.id AND r.player_steam_id = $1 AND r.mode = $2
                 AND r.stage = 0 AND r.is_valid
           )"#,
    )
    .bind(steam_id)
    .bind(mode)
    .fetch_one(pool)
    .await?;
    Ok(n)
}

/// 已完成地图数（带名称搜索）。
pub async fn count_completed_maps_filtered(
    pool: &PgPool,
    steam_id: i64,
    mode: &str,
    query: &str,
) -> Result<i64, sqlx::Error> {
    let (n,): (i64,) = sqlx::query_as(
        r#"SELECT count(*)::BIGINT FROM maps m
           WHERE EXISTS (
               SELECT 1 FROM records r
               WHERE r.map_id = m.id AND r.player_steam_id = $1 AND r.mode = $2
                 AND r.stage = 0 AND r.is_valid
           )
             AND ($3 = '' OR m.name ILIKE '%' || $3 || '%')"#,
    )
    .bind(steam_id)
    .bind(mode)
    .bind(query)
    .fetch_one(pool)
    .await?;
    Ok(n)
}

/// 未完成地图数（带名称搜索）。
pub async fn count_uncompleted_maps(
    pool: &PgPool,
    steam_id: i64,
    mode: &str,
    query: &str,
) -> Result<i64, sqlx::Error> {
    let (n,): (i64,) = sqlx::query_as(
        r#"SELECT count(*)::BIGINT FROM maps m
           WHERE NOT EXISTS (
               SELECT 1 FROM records r
               WHERE r.map_id = m.id AND r.player_steam_id = $1 AND r.mode = $2
                 AND r.stage = 0 AND r.is_valid
           )
             AND ($3 = '' OR m.name ILIKE '%' || $3 || '%')"#,
    )
    .bind(steam_id)
    .bind(mode)
    .bind(query)
    .fetch_one(pool)
    .await?;
    Ok(n)
}

/// 地图总数（不分模式）。
pub async fn count_maps(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let (n,): (i64,) = sqlx::query_as("SELECT count(*)::BIGINT FROM maps")
        .fetch_one(pool)
        .await?;
    Ok(n)
}
