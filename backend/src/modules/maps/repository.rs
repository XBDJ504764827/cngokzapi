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
    wr_player: Option<String>,
    wr_time_ms: Option<i32>,
    wr_date: Option<String>,
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
            wr_player: r.wr_player,
            wr_time: r.wr_time_ms.map(format_duration_ms),
            wr_date: r.wr_date,
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
        r#"SELECT m.id, m.name, m.tier_kzt, m.tier_skz, m.tier_vnl, m.bonuses, m.mappers,
                  m.workshop_id, m.filesize, m.validated,
                  w.wr_player, w.wr_time_ms, w.wr_date
           FROM maps m
           LEFT JOIN LATERAL (
                -- 当前模式主关最快有效成绩 = WR（按 time_ms 升序取一条）
                SELECT p.name                            AS wr_player,
                       r.time_ms                         AS wr_time_ms,
                       to_char(r.created_at, 'YYYY-MM-DD') AS wr_date
                FROM records r
                JOIN players p ON p.steam_id = r.player_steam_id
                WHERE r.map_id = m.id AND r.mode = $3 AND r.stage = 0 AND r.is_valid
                ORDER BY r.time_ms ASC
                LIMIT 1
           ) w ON TRUE
           WHERE ($1 = '' OR m.name ILIKE '%' || $1 || '%')
             AND ($2 = 0 OR CASE $3
                              WHEN 'KZT' THEN m.tier_kzt
                              WHEN 'SKZ' THEN m.tier_skz
                              WHEN 'VNL' THEN m.tier_vnl
                              ELSE m.tier
                            END = $2)
           ORDER BY m.name ASC"#,
    )
    .bind(query)
    .bind(tier)
    .bind(mode)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(MapSummary::from).collect())
}

/// 单张地图；WR 由调用方（详情服务）用主关榜第一名填充，此处不重复查询。
pub async fn get_map(pool: &PgPool, id: i64) -> Result<Option<MapSummary>, sqlx::Error> {
    let row: Option<MapRow> = sqlx::query_as(&format!(
        "SELECT {MAP_COLUMNS}, NULL::text AS wr_player, NULL::int AS wr_time_ms, \
         NULL::text AS wr_date FROM maps WHERE id = $1"
    ))
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
    teleports: i32,
    created_at: String,
}

/// 地图主关榜：`pro = true` 取无存读点（`teleports = 0`），否则取有存读点（`teleports > 0`）。
/// 同一玩家在该分类下取最佳成绩，按时间升序。
pub async fn board_records(
    pool: &PgPool,
    map_id: i64,
    mode: &str,
    pro: bool,
    limit: i64,
) -> Result<Vec<MapRecordEntry>, sqlx::Error> {
    let rows: Vec<RecordRow> = sqlx::query_as(
        r#"SELECT p.name                                  AS player,
                  p.steam_id::TEXT                        AS steam_id,
                  p.avatar_url                            AS avatar,
                  b.time_ms                               AS time_ms,
                  b.points                                AS points,
                  b.teleports                             AS teleports,
                  to_char(b.created_at AT TIME ZONE 'UTC',
                          'YYYY-MM-DD"T"HH24:MI:SS"Z"')      AS created_at
           FROM (
                SELECT DISTINCT ON (r.player_steam_id)
                       r.player_steam_id, r.time_ms, r.points, r.teleports, r.created_at
                FROM records r
                WHERE r.map_id = $1 AND r.mode = $2 AND r.stage = 0 AND r.is_valid
                  AND (($3 AND r.teleports = 0) OR (NOT $3 AND r.teleports > 0))
                ORDER BY r.player_steam_id, r.time_ms ASC
           ) b
           JOIN players p ON p.steam_id = b.player_steam_id
           ORDER BY b.time_ms ASC
           LIMIT $4"#,
    )
    .bind(map_id)
    .bind(mode)
    .bind(pro)
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
            teleports: r.teleports,
            created_at: r.created_at,
        })
        .collect())
}
