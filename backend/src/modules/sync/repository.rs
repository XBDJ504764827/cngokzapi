//! 同步层的数据写入与幂等 upsert（本模块唯一书写 SQL 的地方，见 PROJECT §3.3）。
//!
//! 幂等策略（PROJECT §9.6）：`external_id` 唯一键去重；成绩另加自然键兜底，
//! 撞键时 `ON CONFLICT DO NOTHING`（成绩不可变，重复即视为已存在）。

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// 地图 upsert 载荷（来自 GOKZ.TOP `/maps`）。
pub struct MapUpsert<'a> {
    pub external_id: i64,
    pub name: &'a str,
    pub tier_ovr: i16,
    pub tier_kzt: i16,
    pub tier_skz: i16,
    pub tier_vnl: i16,
    pub bonuses: i16,
    pub workshop_id: Option<i64>,
    pub filesize: i64,
    pub validated: bool,
    pub authors: &'a [String],
    /// 上游创建时间（仅首次插入时写入，避免覆盖本地时间）。
    pub created_at: Option<DateTime<Utc>>,
}

/// 成绩 upsert 载荷（导入与上报共用）。
pub struct RecordUpsert<'a> {
    pub external_id: Option<&'a str>,
    pub steam_id: i64,
    pub map_id: i64,
    pub mode: &'a str,
    pub stage: i32,
    pub tickrate: i32,
    pub teleports: i32,
    pub time_ms: i32,
    pub points: i32,
    pub source: &'a str,
    pub origin_server_id: Option<i64>,
    pub server_name: Option<&'a str>,
    pub server_group: Option<&'a str>,
    pub wr_time_ms: Option<i32>,
    pub is_valid: bool,
    pub created_at: DateTime<Utc>,
}

/// 榜单汇总 upsert 载荷（上游口径字段）。
pub struct ScoreUpsert<'a> {
    pub steam_id: i64,
    pub mode: &'a str,
    pub points: i64,
    pub rank: Option<i32>,
    pub rating: Option<f64>,
    pub maps_completed: i32,
    pub wrs_nub: i32,
    pub wrs_pro: i32,
    pub records_900_plus: i32,
    pub records_800_plus: i32,
    pub region: Option<&'a str>,
}

/// 服务器 upsert 载荷（Steam 发现 / 插件上报共用）。
pub struct ServerUpsert<'a> {
    pub external_id: &'a str,
    pub name: &'a str,
    pub map: &'a str,
    pub ip: &'a str,
    pub port: i32,
    pub player_count: i32,
    pub max_players: i32,
    pub country: Option<&'a str>,
    pub city: Option<&'a str>,
    pub region: Option<&'a str>,
    pub group_name: Option<&'a str>,
    pub group_custom_id: Option<&'a str>,
    pub is_public: bool,
    pub online: bool,
    /// 当前地图难度（按地图名从 `maps` 关联，0 表示未知）。
    pub map_tier: i16,
    /// 实时在线玩家昵称。
    pub player_names: &'a [String],
    /// 心跳时间。
    pub last_seen_at: Option<DateTime<Utc>>,
    /// 来源：`steam` / `plugin`（插件优先，见下）。
    pub source: &'a str,
    /// Steam 服 appid（730 / 4465480）。
    pub appid: Option<i64>,
    /// gametype 原始串（含 KZ 标签，用于只显示 KZ 服）。
    pub game_mode: Option<&'a str>,
    /// Steam 服 id。
    pub steam_id: Option<&'a str>,
}

/// 封禁 upsert 载荷。
pub struct BanUpsert<'a> {
    pub external_id: &'a str,
    pub steam_id: i64,
    pub name: &'a str,
    pub avatar: &'a str,
    pub reason: &'a str,
    pub server: &'a str,
    pub detail: &'a str,
    pub banned_at: DateTime<Utc>,
    pub ban_type: Option<&'a str>,
    pub expires_at: Option<DateTime<Utc>>,
    pub source: &'a str,
}

/// 地图批量 upsert：整批一次往返（`jsonb_to_recordset`），避免逐行 1100 次往返。
pub async fn upsert_maps_batch(
    pool: &PgPool,
    rows: &[MapUpsert<'_>],
) -> Result<usize, sqlx::Error> {
    if rows.is_empty() {
        return Ok(0);
    }
    #[derive(serde::Serialize)]
    struct Row<'a> {
        external_id: i64,
        name: &'a str,
        tier_ovr: i16,
        tier_kzt: i16,
        tier_skz: i16,
        tier_vnl: i16,
        bonuses: i16,
        workshop_id: Option<i64>,
        filesize: i64,
        validated: bool,
        authors: &'a [String],
        created_at: Option<DateTime<Utc>>,
    }
    let payload = serde_json::to_string(
        &rows
            .iter()
            .map(|m| Row {
                external_id: m.external_id,
                name: m.name,
                tier_ovr: m.tier_ovr,
                tier_kzt: m.tier_kzt,
                tier_skz: m.tier_skz,
                tier_vnl: m.tier_vnl,
                bonuses: m.bonuses,
                workshop_id: m.workshop_id,
                filesize: m.filesize,
                validated: m.validated,
                authors: m.authors,
                created_at: m.created_at,
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    sqlx::query(
        r#"INSERT INTO maps (external_id, name, tier, mode, tier_kzt, tier_skz, tier_vnl,
                             bonuses, workshop_id, filesize, validated, authors, synced_at, created_at)
           SELECT u.external_id, u.name, u.tier_ovr, 'OVR', u.tier_kzt, u.tier_skz, u.tier_vnl,
                  u.bonuses, u.workshop_id, u.filesize, u.validated, u.authors, now(),
                  COALESCE(u.created_at, now())
           FROM jsonb_to_recordset($1::jsonb) AS u(
                external_id BIGINT, name TEXT, tier_ovr SMALLINT, tier_kzt SMALLINT,
                tier_skz SMALLINT, tier_vnl SMALLINT, bonuses SMALLINT, workshop_id BIGINT,
                filesize BIGINT, validated BOOLEAN, authors TEXT[], created_at TIMESTAMPTZ)
           ON CONFLICT (name) DO UPDATE SET
               external_id = EXCLUDED.external_id,
               tier        = EXCLUDED.tier,
               tier_kzt    = EXCLUDED.tier_kzt,
               tier_skz    = EXCLUDED.tier_skz,
               tier_vnl    = EXCLUDED.tier_vnl,
               bonuses     = EXCLUDED.bonuses,
               workshop_id = EXCLUDED.workshop_id,
               filesize    = EXCLUDED.filesize,
               validated   = EXCLUDED.validated,
               authors     = EXCLUDED.authors,
               synced_at   = now()"#,
    )
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(rows.len())
}

/// 确保地图存在（最新记录里出现的新图先建最小行），返回内部 id。
pub async fn ensure_map(
    pool: &PgPool,
    external_id: i64,
    name: &str,
    tier: i16,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO maps (external_id, name, tier, mode, validated)
           VALUES ($1, $2, $3, 'OVR', TRUE)
           ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
           RETURNING id"#,
    )
    .bind(external_id)
    .bind(name)
    .bind(tier)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// 按地图名取当前难度（`tier_kzt` 优先，回退综合 tier）；无此图返回 None。
pub async fn map_tier_by_name(pool: &PgPool, name: &str) -> Result<Option<i16>, sqlx::Error> {
    let row: Option<(i16,)> = sqlx::query_as(
        "SELECT CASE WHEN tier_kzt > 0 THEN tier_kzt ELSE tier END FROM maps WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

/// 按地图名取内部 id。
pub async fn map_id_by_name(pool: &PgPool, name: &str) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT id FROM maps WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

/// 批量写入地图制作人显示名（来自 kzgo 地图元数据；缺失的图保持空数组）。
pub async fn update_mappers(
    pool: &PgPool,
    rows: &[(String, Vec<String>)],
) -> Result<usize, sqlx::Error> {
    if rows.is_empty() {
        return Ok(0);
    }
    #[derive(serde::Serialize)]
    struct Row {
        name: String,
        mappers: Vec<String>,
    }
    let payload = serde_json::to_string(
        &rows
            .iter()
            .map(|(name, mappers)| Row {
                name: name.clone(),
                mappers: mappers.clone(),
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    let res = sqlx::query(
        r#"UPDATE maps m SET mappers = u.mappers
           FROM jsonb_to_recordset($1::jsonb) AS u(name TEXT, mappers TEXT[])
           WHERE m.name = u.name"#,
    )
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() as usize)
}

/// 地图名 → 综合 tier（用于服务器当前地图难度）。
pub async fn map_tiers(
    pool: &PgPool,
) -> Result<std::collections::HashMap<String, i16>, sqlx::Error> {
    let rows: Vec<(String, i16)> =
        sqlx::query_as("SELECT name, CASE WHEN tier_kzt > 0 THEN tier_kzt ELSE tier END FROM maps")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().collect())
}

/// 地图 bonus 数量（决定要同步哪些 stage）。
pub async fn map_bonuses(pool: &PgPool, name: &str) -> Result<i16, sqlx::Error> {
    let row: Option<(i16,)> = sqlx::query_as("SELECT bonuses FROM maps WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0).unwrap_or(0))
}

/// 玩家档案 upsert（国家 / 头像来自 GOKZ.TOP `/players/{steamid64}`）。
/// 空值视为「本次没拿到」，不覆盖库里已有的值。
pub async fn upsert_player_profile(
    pool: &PgPool,
    steam_id: i64,
    name: &str,
    avatar_url: Option<&str>,
    country: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO players (steam_id, name, avatar_url, country)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (steam_id) DO UPDATE SET
               name = CASE WHEN EXCLUDED.name = '' THEN players.name ELSE EXCLUDED.name END,
               avatar_url = COALESCE(NULLIF(EXCLUDED.avatar_url, ''), players.avatar_url),
               country    = COALESCE(NULLIF(EXCLUDED.country, ''), players.country),
               avatar_synced_at = CASE
                   WHEN EXCLUDED.avatar_url IS NOT NULL AND EXCLUDED.avatar_url <> ''
                   THEN now() ELSE players.avatar_synced_at END,
               updated_at = now()"#,
    )
    .bind(steam_id)
    .bind(name)
    // players.avatar_url 为 NOT NULL DEFAULT ''：无头像时写空串（ON CONFLICT 前会校验非空）
    .bind(avatar_url.unwrap_or(""))
    .bind(country)
    .execute(pool)
    .await?;
    Ok(())
}

/// 玩家 upsert；空昵称不覆盖已有昵称。
pub async fn upsert_player(pool: &PgPool, steam_id: i64, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO players (steam_id, name) VALUES ($1, $2)
           ON CONFLICT (steam_id) DO UPDATE SET
               name = CASE WHEN EXCLUDED.name = '' THEN players.name ELSE EXCLUDED.name END,
               updated_at = now()"#,
    )
    .bind(steam_id)
    .bind(name)
    .execute(pool)
    .await?;
    Ok(())
}

/// 成绩 upsert；返回是否新插入（false = 已存在，即幂等命中）。
pub async fn upsert_record(pool: &PgPool, r: &RecordUpsert<'_>) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        r#"INSERT INTO records (player_steam_id, map_id, mode, time_ms, points, created_at,
                                source, external_id, stage, tickrate, teleports,
                                origin_server_id, server_name, server_group, wr_time_ms,
                                is_valid, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, now())
           ON CONFLICT DO NOTHING"#,
    )
    .bind(r.steam_id)
    .bind(r.map_id)
    .bind(r.mode)
    .bind(r.time_ms)
    .bind(r.points)
    .bind(r.created_at)
    .bind(r.source)
    .bind(r.external_id)
    .bind(r.stage)
    .bind(r.tickrate)
    .bind(r.teleports)
    .bind(r.origin_server_id)
    .bind(r.server_name)
    .bind(r.server_group)
    .bind(r.wr_time_ms)
    .bind(r.is_valid)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// 带状态更新的成绩 upsert（用于 gokz.top `updated_since` 增量）：
/// 已存在的记录会更新 `is_valid` / `points` / `updated_at`，以支持上游软删除与改分。
/// 返回是否有行被插入或更新。
pub async fn upsert_record_state(pool: &PgPool, r: &RecordUpsert<'_>) -> Result<bool, sqlx::Error> {
    let Some(external_id) = r.external_id else {
        return upsert_record(pool, r).await;
    };
    // 先按 external_id 或自然键定位已有行：同一成绩可能已由另一来源（KZTimer / gokz）写入，
    // 直接 INSERT 会撞 `uq_records_natural`。命中则只更新状态，未命中才插入。
    let existing: Option<(i64,)> = sqlx::query_as(
        r#"SELECT id FROM records
           WHERE external_id = $1
              OR (player_steam_id = $2 AND map_id = $3 AND mode = $4 AND stage = $5
                  AND tickrate = $6 AND teleports = $7 AND time_ms = $8 AND created_at = $9)
           LIMIT 1"#,
    )
    .bind(external_id)
    .bind(r.steam_id)
    .bind(r.map_id)
    .bind(r.mode)
    .bind(r.stage)
    .bind(r.tickrate)
    .bind(r.teleports)
    .bind(r.time_ms)
    .bind(r.created_at)
    .fetch_optional(pool)
    .await?;

    if let Some((id,)) = existing {
        sqlx::query(
            "UPDATE records SET is_valid = $2, points = $3, updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .bind(r.is_valid)
        .bind(r.points)
        .execute(pool)
        .await?;
        return Ok(true);
    }

    upsert_record(pool, r).await
}

/// 读取增量游标（不存在时返回 None）。
pub async fn get_sync_state(pool: &PgPool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM sync_state WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

/// 写入增量游标。
pub async fn set_sync_state(pool: &PgPool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO sync_state (key, value, updated_at) VALUES ($1, $2, now())
           ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now()"#,
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

/// 封禁级联：把该玩家所有成绩标为无效，并移除榜单汇总（不再出现在排行榜/地图榜）。
/// 返回被失效的成绩条数。
pub async fn invalidate_player_records(pool: &PgPool, steam_id: i64) -> Result<u64, sqlx::Error> {
    let records = sqlx::query(
        "UPDATE records SET is_valid = FALSE, updated_at = now() \
         WHERE player_steam_id = $1 AND is_valid",
    )
    .bind(steam_id)
    .execute(pool)
    .await?
    .rows_affected();
    sqlx::query("DELETE FROM player_scores WHERE player_steam_id = $1")
        .bind(steam_id)
        .execute(pool)
        .await?;
    Ok(records)
}

/// 榜单汇总 upsert（不覆盖 `points_local`，由计分单独写入）。
pub async fn upsert_score(pool: &PgPool, s: &ScoreUpsert<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO player_scores (player_steam_id, mode, points, rank, rating, maps_completed,
                                      wrs_nub, wrs_pro, records_900_plus, records_800_plus, source, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'gokz_top', now())
           ON CONFLICT (player_steam_id, mode) DO UPDATE SET
               points           = EXCLUDED.points,
               rank             = EXCLUDED.rank,
               rating           = EXCLUDED.rating,
               maps_completed   = EXCLUDED.maps_completed,
               wrs_nub          = EXCLUDED.wrs_nub,
               wrs_pro          = EXCLUDED.wrs_pro,
               records_900_plus = EXCLUDED.records_900_plus,
               records_800_plus = EXCLUDED.records_800_plus,
               source           = EXCLUDED.source,
               updated_at       = now()"#,
    )
    .bind(s.steam_id)
    .bind(s.mode)
    .bind(s.points)
    .bind(s.rank)
    .bind(s.rating)
    .bind(s.maps_completed)
    .bind(s.wrs_nub)
    .bind(s.wrs_pro)
    .bind(s.records_900_plus)
    .bind(s.records_800_plus)
    .execute(pool)
    .await?;
    // 区域信息只在上游给出时补齐
    if let Some(region) = s.region {
        sqlx::query("UPDATE players SET region = $2, updated_at = now() WHERE steam_id = $1")
            .bind(s.steam_id)
            .bind(region)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// 榜单批量导入载荷（每页 100 条，避免逐行往返）。
pub struct LeaderboardRow<'a> {
    pub steam_id: i64,
    pub name: &'a str,
    pub points: i64,
    pub rank: Option<i32>,
    pub rating: Option<f64>,
    pub maps_completed: i32,
    pub wrs_nub: i32,
    pub wrs_pro: i32,
    pub records_900_plus: i32,
    pub records_800_plus: i32,
}

/// 榜单批量 upsert：一次写入 players 与 player_scores（全量导入 / 10 分钟刷新共用）。
pub async fn upsert_leaderboard_batch(
    pool: &PgPool,
    mode: &str,
    rows: &[LeaderboardRow<'_>],
) -> Result<usize, sqlx::Error> {
    if rows.is_empty() {
        return Ok(0);
    }
    #[derive(serde::Serialize)]
    struct Row<'a> {
        steam_id: i64,
        name: &'a str,
        points: i64,
        rank: Option<i32>,
        rating: Option<f64>,
        maps_completed: i32,
        wrs_nub: i32,
        wrs_pro: i32,
        records_900_plus: i32,
        records_800_plus: i32,
    }
    let payload = serde_json::to_string(
        &rows
            .iter()
            .map(|r| Row {
                steam_id: r.steam_id,
                name: r.name,
                points: r.points,
                rank: r.rank,
                rating: r.rating,
                maps_completed: r.maps_completed,
                wrs_nub: r.wrs_nub,
                wrs_pro: r.wrs_pro,
                records_900_plus: r.records_900_plus,
                records_800_plus: r.records_800_plus,
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    sqlx::query(
        r#"WITH data AS (
               SELECT * FROM jsonb_to_recordset($1::jsonb) AS x(
                   steam_id BIGINT, name TEXT, points BIGINT, rank INTEGER,
                   rating DOUBLE PRECISION, maps_completed INTEGER,
                   wrs_nub INTEGER, wrs_pro INTEGER,
                   records_900_plus INTEGER, records_800_plus INTEGER
               )
           ),
           upsert_players AS (
               INSERT INTO players (steam_id, name)
               SELECT steam_id, name FROM data
               ON CONFLICT (steam_id) DO UPDATE SET
                   name = CASE WHEN EXCLUDED.name = '' THEN players.name ELSE EXCLUDED.name END,
                   updated_at = now()
           )
           INSERT INTO player_scores (player_steam_id, mode, points, rank, rating, maps_completed,
                                      wrs_nub, wrs_pro, records_900_plus, records_800_plus,
                                      source, updated_at)
           SELECT steam_id, $2, points, rank, rating, maps_completed, wrs_nub, wrs_pro,
                  records_900_plus, records_800_plus, 'gokz_top', now()
           FROM data
           ON CONFLICT (player_steam_id, mode) DO UPDATE SET
               points           = EXCLUDED.points,
               rank             = EXCLUDED.rank,
               rating           = EXCLUDED.rating,
               maps_completed   = EXCLUDED.maps_completed,
               wrs_nub          = EXCLUDED.wrs_nub,
               wrs_pro          = EXCLUDED.wrs_pro,
               records_900_plus = EXCLUDED.records_900_plus,
               records_800_plus = EXCLUDED.records_800_plus,
               source           = EXCLUDED.source,
               updated_at       = now()"#,
    )
    .bind(payload)
    .bind(mode)
    .execute(pool)
    .await?;
    Ok(rows.len())
}

/// 写回「本站汇总分」（计分引擎结果，PROJECT §9.4）。
pub async fn update_local_points(
    pool: &PgPool,
    steam_id: i64,
    mode: &str,
    points_local: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO player_scores (player_steam_id, mode, points_local, source, updated_at)
           VALUES ($1, $2, $3, 'local', now())
           ON CONFLICT (player_steam_id, mode) DO UPDATE SET
               points_local = EXCLUDED.points_local, updated_at = now()"#,
    )
    .bind(steam_id)
    .bind(mode)
    .bind(points_local)
    .execute(pool)
    .await?;
    Ok(())
}

/// Steam 服务器 upsert：以 `(ip, port)` 合并；插件数据（10 分钟内）优先，不覆盖。
/// 同步开始前先调 `mark_steam_servers_offline`，不在本轮列表里的 Steam 服标离线。
pub async fn upsert_steam_server(pool: &PgPool, s: &ServerUpsert<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO servers (external_id, name, map, ip, port, player_count, max_players, updated_at,
                                country, city, region, group_name, group_custom_id, is_public, online,
                                map_tier, player_names, last_seen_at, source, appid, game_mode, steam_id, reported_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, now(), $8, $9, $10, $11, $12, $13, $14, $15, $16,
                   COALESCE($17, now()), 'steam', $18, $19, $20, now())
           ON CONFLICT (ip, port) DO UPDATE SET
               external_id     = EXCLUDED.external_id,
               name            = EXCLUDED.name,
               map             = EXCLUDED.map,
               port            = EXCLUDED.port,
               player_count    = EXCLUDED.player_count,
               max_players     = EXCLUDED.max_players,
               country         = EXCLUDED.country,
               city            = EXCLUDED.city,
               region          = EXCLUDED.region,
               group_name      = EXCLUDED.group_name,
               group_custom_id = EXCLUDED.group_custom_id,
               is_public       = EXCLUDED.is_public,
               online          = EXCLUDED.online,
               map_tier        = EXCLUDED.map_tier,
               player_names    = EXCLUDED.player_names,
               source          = 'steam',
               appid           = EXCLUDED.appid,
               game_mode       = EXCLUDED.game_mode,
               steam_id        = EXCLUDED.steam_id,
               updated_at      = now(),
               last_seen_at    = COALESCE(EXCLUDED.last_seen_at, now()),
               reported_at     = now()
           WHERE servers.source <> 'plugin' OR servers.reported_at IS NULL
                 OR servers.reported_at < now() - interval '10 minutes'"#,
    )
    .bind(s.external_id)
    .bind(s.name)
    .bind(s.map)
    .bind(s.ip)
    .bind(s.port)
    .bind(s.player_count)
    .bind(s.max_players)
    .bind(s.country)
    .bind(s.city)
    .bind(s.region)
    .bind(s.group_name)
    .bind(s.group_custom_id)
    .bind(s.is_public)
    .bind(s.online)
    .bind(s.map_tier)
    .bind(s.player_names)
    .bind(s.last_seen_at)
    .bind(s.appid)
    .bind(s.game_mode)
    .bind(s.steam_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// 插件服务器上报 upsert：以 `(ip, port)` 合并，始终以 `source='plugin'` 覆盖（实时数据最优）。
pub async fn upsert_plugin_server(pool: &PgPool, s: &ServerUpsert<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO servers (external_id, name, map, ip, port, player_count, max_players, updated_at,
                                is_public, online, map_tier, player_names, last_seen_at,
                                source, game_mode, reported_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, now(), TRUE, TRUE, $8, $9, now(), 'plugin', $10, now())
           ON CONFLICT (ip, port) DO UPDATE SET
               external_id  = EXCLUDED.external_id,
               name         = EXCLUDED.name,
               map          = EXCLUDED.map,
               player_count = EXCLUDED.player_count,
               max_players  = EXCLUDED.max_players,
               map_tier     = EXCLUDED.map_tier,
               player_names = EXCLUDED.player_names,
               online       = TRUE,
               is_public    = TRUE,
               source       = 'plugin',
               game_mode    = COALESCE(EXCLUDED.game_mode, servers.game_mode),
               updated_at   = now(),
               last_seen_at = now(),
               reported_at  = now()"#,
    )
    .bind(s.external_id)
    .bind(s.name)
    .bind(s.map)
    .bind(s.ip)
    .bind(s.port)
    .bind(s.player_count)
    .bind(s.max_players)
    .bind(s.map_tier)
    .bind(s.player_names)
    .bind(s.game_mode)
    .execute(pool)
    .await?;
    Ok(())
}

/// 把上一轮由 Steam 标记为在线的服务器先置为离线（本轮重新发现的会再写回在线）。
pub async fn mark_steam_servers_offline(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE servers SET online = FALSE, updated_at = now() WHERE source = 'steam' AND online",
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

/// 封禁 upsert（按 external_id 幂等）。
pub async fn upsert_ban(pool: &PgPool, b: &BanUpsert<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO bans (steam_id, name, avatar_url, reason, server, detail, banned_at,
                             external_id, ban_type, expires_at, source)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           ON CONFLICT (external_id) DO UPDATE SET
               name       = EXCLUDED.name,
               reason     = EXCLUDED.reason,
               server     = EXCLUDED.server,
               detail     = EXCLUDED.detail,
               banned_at  = EXCLUDED.banned_at,
               ban_type   = EXCLUDED.ban_type,
               expires_at = EXCLUDED.expires_at,
               source     = EXCLUDED.source"#,
    )
    .bind(b.steam_id)
    .bind(b.name)
    .bind(b.avatar)
    .bind(b.reason)
    .bind(b.server)
    .bind(b.detail)
    .bind(b.banned_at)
    .bind(b.external_id)
    .bind(b.ban_type)
    .bind(b.expires_at)
    .bind(b.source)
    .execute(pool)
    .await?;
    Ok(())
}

/// 各表计数（供启动种子判断是否为空）。
pub async fn table_count(pool: &PgPool, table: &str) -> Result<i64, sqlx::Error> {
    // table 只来自本模块内部的常量，不接受外部输入
    let sql = match table {
        "maps" => "SELECT count(*) FROM maps",
        "records" => "SELECT count(*) FROM records",
        "servers" => "SELECT count(*) FROM servers",
        "bans" => "SELECT count(*) FROM bans",
        "player_scores" => "SELECT count(*) FROM player_scores",
        other => return Err(sqlx::Error::Protocol(format!("未支持的计数表：{other}"))),
    };
    let row: (i64,) = sqlx::query_as(sql).fetch_one(pool).await?;
    Ok(row.0)
}
