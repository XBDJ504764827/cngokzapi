//! 地图数据源：读本站 `maps` 与 `records`（由 `modules/sync` 同步）。

use sqlx::{FromRow, PgPool};

use super::models::{MapRecordEntry, MapSummary};
use crate::modules::format::format_duration_ms;

#[derive(FromRow)]
struct MapRow {
    id: i64,
    name: String,
    tier_kzt: i16,
    tier_skz: i16,
    tier_vnl: i16,
    bonuses: i16,
    mappers: Vec<String>,
    workshop_id: Option<i64>,
    filesize: i64,
    validated: bool,
}

impl From<MapRow> for MapSummary {
    fn from(r: MapRow) -> Self {
        MapSummary {
            id: r.id,
            name: r.name,
            tier_kzt: r.tier_kzt,
            tier_skz: r.tier_skz,
            tier_vnl: r.tier_vnl,
            bonuses: r.bonuses,
            mappers: r.mappers,
            workshop_id: r.workshop_id,
            filesize: r.filesize,
            validated: r.validated,
        }
    }
}

const MAP_COLUMNS: &str = "id, name, tier_kzt, tier_skz, tier_vnl, bonuses, mappers, \
     workshop_id, filesize, validated";

/// 地图列表；`query` 为名称模糊匹配，`tier` 为 0 时不过滤难度，
/// `mode` 决定按哪个模式的 tier 过滤（`OVR` 用综合 tier）。
pub async fn list_maps(
    pool: &PgPool,
    query: &str,
    tier: i16,
    mode: &str,
) -> Result<Vec<MapSummary>, sqlx::Error> {
    let rows: Vec<MapRow> = sqlx::query_as(
        r#"SELECT id, name, tier_kzt, tier_skz, tier_vnl, bonuses, mappers,
                  workshop_id, filesize, validated
           FROM maps
           WHERE ($1 = '' OR name ILIKE '%' || $1 || '%')
             AND ($2 = 0 OR CASE $3
                              WHEN 'KZT' THEN tier_kzt
                              WHEN 'SKZ' THEN tier_skz
                              WHEN 'VNL' THEN tier_vnl
                              ELSE tier
                            END = $2)
           ORDER BY name ASC"#,
    )
    .bind(query)
    .bind(tier)
    .bind(mode)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(MapSummary::from).collect())
}

/// 单张地图。
pub async fn get_map(pool: &PgPool, id: i64) -> Result<Option<MapSummary>, sqlx::Error> {
    let row: Option<MapRow> =
        sqlx::query_as(&format!("SELECT {MAP_COLUMNS} FROM maps WHERE id = $1"))
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(MapSummary::from))
}

#[derive(FromRow)]
struct RecordRow {
    player: String,
    steam_id: String,
    avatar: Option<String>,
    time_ms: i32,
    points: i32,
    date: String,
}

/// 地图主关榜：同一玩家取最佳成绩，按时间升序。
pub async fn top_records(
    pool: &PgPool,
    map_id: i64,
    mode: &str,
    limit: i64,
) -> Result<Vec<MapRecordEntry>, sqlx::Error> {
    let rows: Vec<RecordRow> = sqlx::query_as(
        r#"SELECT p.name                                  AS player,
                  p.steam_id::TEXT                        AS steam_id,
                  p.avatar_url                            AS avatar,
                  b.time_ms                               AS time_ms,
                  b.points                                AS points,
                  to_char(b.created_at, 'YYYY-MM-DD')      AS date
           FROM (
                SELECT DISTINCT ON (r.player_steam_id)
                       r.player_steam_id, r.time_ms, r.points, r.created_at
                FROM records r
                WHERE r.map_id = $1 AND r.mode = $2 AND r.stage = 0 AND r.is_valid
                ORDER BY r.player_steam_id, r.time_ms ASC
           ) b
           JOIN players p ON p.steam_id = b.player_steam_id
           ORDER BY b.time_ms ASC
           LIMIT $3"#,
    )
    .bind(map_id)
    .bind(mode)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| MapRecordEntry {
            rank: (i + 1) as u32,
            player: r.player,
            steam_id64: r.steam_id,
            avatar: r.avatar.unwrap_or_default(),
            time: format_duration_ms(r.time_ms),
            points: i64::from(r.points),
            date: r.date,
        })
        .collect())
}
