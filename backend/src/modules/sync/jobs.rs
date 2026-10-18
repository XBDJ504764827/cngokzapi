//! 各类同步任务的实现：外部拉取 → 字段映射 → 幂等落库。
//!
//! 每个任务都是「可重入」的：重复执行只会更新已有行或跳过（PROJECT §9.6）。

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::repository::{self, BanUpsert, MapUpsert, RecordUpsert, ScoreUpsert, ServerUpsert};
use crate::{
    modules::scoring,
    sources::{
        gokz_top::{GokzTop, TopRecordRow},
        kztimer::{self, Kztimer},
        steam::Steam,
        SourceClient, SourceError,
    },
};

/// 全站三个模式（与前端 `types/common.ts` 一致）。
pub const MODES: [&str; 3] = ["KZT", "SKZ", "VNL"];

/// 单次成绩拉取上限（避免对上游造成压力；上游该接口不支持 offset）。
const MAP_PAGE: i64 = 1000;

/// 同步任务错误：上游不可用或数据库异常。
#[derive(Debug)]
pub enum JobError {
    Source(SourceError),
    Db(sqlx::Error),
}

impl std::fmt::Display for JobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobError::Source(e) => write!(f, "{e}"),
            JobError::Db(e) => write!(f, "数据库异常：{e}"),
        }
    }
}

impl From<SourceError> for JobError {
    fn from(e: SourceError) -> Self {
        JobError::Source(e)
    }
}

impl From<sqlx::Error> for JobError {
    fn from(e: sqlx::Error) -> Self {
        JobError::Db(e)
    }
}

/// RFC3339 → UTC；解析失败退化为当前时间（记录仍可入库）。
fn parse_time(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// KZTimer 的时间无时区（如 `2026-10-18T15:27:44`，实为 UTC），补充 UTC 后解析；失败退化为当前时间。
fn parse_naive_utc(s: &str) -> DateTime<Utc> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// 秒（浮点）→ 毫秒整数。
fn secs_to_ms(v: f64) -> i32 {
    (v * 1000.0).round() as i32
}

/// 上游模式名：字符串优先，缺失时用 `mode_id` 兜底（200=KZT / 201=SKZ / 202=VNL）。
fn mode_name(name: Option<&str>, id: Option<i32>) -> String {
    if let Some(n) = name {
        if !n.trim().is_empty() {
            return n.to_uppercase();
        }
    }
    match id {
        Some(201) => "SKZ".to_string(),
        Some(202) => "VNL".to_string(),
        _ => "KZT".to_string(),
    }
}

// ------------------------------ 地图库 ------------------------------

pub async fn run_maps(pool: &PgPool, src: &SourceClient) -> Result<usize, JobError> {
    let rows = GokzTop(src).maps().await?;
    let payload: Vec<MapUpsert<'_>> = rows
        .iter()
        .map(|m| MapUpsert {
            external_id: m.id,
            name: &m.name,
            tier_ovr: m.tiers.ovr,
            tier_kzt: m.tiers.kzt,
            tier_skz: m.tiers.skz,
            tier_vnl: m.tiers.vnl,
            bonuses: m.bonus_count,
            workshop_id: m.workshop_id,
            filesize: m.filesize,
            validated: m.validated,
            authors: &m.authors,
            created_at: m.created_on.as_deref().map(parse_time),
        })
        .collect();
    let n = repository::upsert_maps_batch(pool, &payload).await?;

    // 制作人显示名来自 kzgo 元数据（gokz.top 只给作者 SteamID）；失败只告警，
    // 下次定时任务会再补，不阻塞地图主流程。
    match crate::sources::kzgo::Kzgo(src).maps().await {
        Ok(list) => {
            // 上游 mapperNames 可能含 null，这里过滤掉
            let rows: Vec<(String, Vec<String>)> = list
                .into_iter()
                .map(|m| (m.name, m.mapper_names.into_iter().flatten().collect()))
                .collect();
            let updated = repository::update_mappers(pool, &rows).await?;
            tracing::info!(updated, "已补写地图制作人");
        }
        Err(e) => tracing::warn!(error = %e, "kzgo 地图元数据不可用，制作人暂缺"),
    }
    Ok(n)
}

// ------------------------------ 排行榜 ------------------------------

/// 上游榜单单页上限（`limit <= 100`，超出直接 422）。
const LEADERBOARD_PAGE: i64 = 100;
/// 每个模式为前 N 名额外入队「玩家档案」同步，补齐记录数/最近游玩/本地汇总分。
const ENRICH_TOP: usize = 60;

/// 拉取某模式玩家榜前 `limit` 名（自动按 100 分页），写 players + player_scores。
pub async fn run_leaderboard(
    pool: &PgPool,
    src: &SourceClient,
    mode: &str,
    limit: i64,
) -> Result<usize, JobError> {
    let client = GokzTop(src);
    let mut n = 0;
    let mut offset = 0i64;
    let mut enrich: Vec<String> = Vec::new();
    while offset < limit {
        let want = LEADERBOARD_PAGE.min(limit - offset);
        let page = client.leaderboard(mode, want, offset).await?;
        if let Some(total) = page.count {
            tracing::debug!(mode, upstream_total = total, "上游榜单总人数");
        }
        let got = page.data.len() as i64;
        for row in &page.data {
            let Some(steam_id) = row.player.steamid64.parse::<i64>().ok() else {
                continue;
            };
            if enrich.len() < ENRICH_TOP {
                enrich.push(row.player.steamid64.clone());
            }
            repository::upsert_player(pool, steam_id, &row.player.display_name).await?;
            repository::upsert_score(
                pool,
                &ScoreUpsert {
                    steam_id,
                    mode,
                    points: row.points,
                    rank: row.rank.or(row.global_rank),
                    rating: row.rating,
                    maps_completed: row.unique_map_finishes,
                    wrs_nub: row.wrs_nub,
                    wrs_pro: row.wrs_pro,
                    records_900_plus: row.records_900_plus,
                    records_800_plus: row.records_800_plus,
                    region: None,
                },
            )
            .await?;
            n += 1;
        }
        offset += got;
        if got < want || got == 0 {
            break;
        }
    }
    // 榜单前 N 名逐个补齐档案（按需回源队列，幂等）
    for steamid64 in enrich {
        super::queue::enqueue(pool, super::KIND_PLAYER, &steamid64).await?;
    }
    Ok(n)
}

// ------------------------------ 最新记录（KZTimer 实时广播源） ------------------------------

/// KZTimer 拉取条数（接口不支持按模式过滤，拉回后本地筛选）。
const RECENT_FETCH: i64 = 300;

pub async fn run_recent(pool: &PgPool, src: &SourceClient, mode: &str) -> Result<usize, JobError> {
    let rows = Kztimer(src).recent_records(RECENT_FETCH).await?;
    let mut n = 0;
    for r in &rows {
        // 接口不支持按模式过滤，这里本地筛选（PROJECT §9.2）
        if kztimer::mode_name(&r.mode) != Some(mode) {
            continue;
        }
        let Some(steam_id) = r.steamid64.parse::<i64>().ok() else {
            continue;
        };
        let Some(map_name) = r.map_name.as_deref() else {
            continue;
        };
        repository::upsert_player(pool, steam_id, &r.player_name).await?;
        let map_id = match repository::map_id_by_name(pool, map_name).await? {
            Some(id) => id,
            None => repository::ensure_map(pool, r.map_id.unwrap_or(0), map_name, 0).await?,
        };
        let external_id = format!("kztimer:{}", r.id);
        repository::upsert_record(
            pool,
            &RecordUpsert {
                external_id: Some(&external_id),
                steam_id,
                map_id,
                mode,
                stage: r.stage,
                tickrate: if r.tickrate > 0 { r.tickrate } else { 128 },
                teleports: r.teleports,
                time_ms: secs_to_ms(r.time),
                points: r.points,
                source: "kztimer",
                origin_server_id: r.server_id,
                server_name: r.server_name.as_deref(),
                server_group: None,
                wr_time_ms: None,
                is_valid: true,
                created_at: parse_naive_utc(&r.created_on),
            },
        )
        .await?;
        n += 1;
    }
    Ok(n)
}

// ------------------------------ 单张地图（按需整图同步） ------------------------------

/// 拉取某地图某模式的全部主关 + bonus 成绩（分页上限见常量）。
pub async fn run_map(
    pool: &PgPool,
    src: &SourceClient,
    map_name: &str,
    mode: &str,
) -> Result<usize, JobError> {
    let Some(map_id) = repository::map_id_by_name(pool, map_name).await? else {
        return Err(JobError::Db(sqlx::Error::RowNotFound));
    };
    let bonuses = repository::map_bonuses(pool, map_name).await?;
    let client = GokzTop(src);
    let mut n = 0;
    // 上游 `/records?map_name=&stage=&limit=` 只支持 limit（无 offset），
    // 故每个关卡一次取前 MAP_PAGE 条。
    for stage in 0..=bonuses as i32 {
        let page = client.map_records(map_name, stage, MAP_PAGE).await?;
        n += write_top_records(pool, map_id, mode, &page.data).await?;
    }
    Ok(n)
}

/// 把 `/records` 形态的成绩写入库（只保留目标模式）。
async fn write_top_records(
    pool: &PgPool,
    map_id: i64,
    mode: &str,
    rows: &[TopRecordRow],
) -> Result<usize, JobError> {
    let mut n = 0;
    for r in rows {
        let row_mode = mode_name(r.mode.as_deref(), r.mode_id);
        if !row_mode.eq_ignore_ascii_case(mode) {
            continue;
        }
        let Some(steam_id) = r.player.steamid64.parse::<i64>().ok() else {
            continue;
        };
        repository::upsert_player(pool, steam_id, &r.player.display_name).await?;
        repository::upsert_record(
            pool,
            &RecordUpsert {
                external_id: Some(&r.uuid),
                steam_id,
                map_id,
                mode,
                stage: r.stage,
                tickrate: if r.tickrate > 0 { r.tickrate } else { 128 },
                teleports: r.teleports,
                time_ms: secs_to_ms(r.time),
                points: r.points,
                source: "gokz_top",
                origin_server_id: r.server_id,
                server_name: r.server_name.as_deref(),
                server_group: None,
                wr_time_ms: r.wr_time.map(secs_to_ms),
                is_valid: r.is_valid.unwrap_or(true),
                created_at: parse_time(&r.created_on),
            },
        )
        .await?;
        n += 1;
    }
    Ok(n)
}

// ------------------------------ 单个玩家（按需懒加载） ------------------------------

/// 拉取某玩家三模式档案 + 成绩，并写入本站汇总分。
pub async fn run_player(
    pool: &PgPool,
    src: &SourceClient,
    steamid64: &str,
) -> Result<usize, JobError> {
    let Some(steam_id) = steamid64.parse::<i64>().ok() else {
        return Err(JobError::Db(sqlx::Error::RowNotFound));
    };
    let client = GokzTop(src);
    let mut n = 0;

    // 1) 三模式档案（排名/积分/完成图数等上游口径）
    for mode in MODES {
        if let Some(p) = client.player(steamid64, mode).await? {
            repository::upsert_player(pool, steam_id, &p.player.display_name).await?;
            repository::upsert_score(
                pool,
                &ScoreUpsert {
                    steam_id,
                    mode,
                    points: p.points,
                    rank: p.rank.or(p.global_rank),
                    rating: p.rating,
                    maps_completed: p.unique_map_finishes,
                    wrs_nub: p.wrs_nub,
                    wrs_pro: p.wrs_pro,
                    records_900_plus: p.records_900_plus,
                    records_800_plus: p.records_800_plus,
                    region: p.region.as_deref(),
                },
            )
            .await?;
            n += 1;
        }
    }

    // 2) 成绩（用于地图榜、个人记录与本站汇总分）
    let page = client.player_records(steamid64, 1000).await?;
    for r in &page.data {
        let (Some(map_name), Some(map_ext_id)) = (r.map_name.as_deref(), r.map_id) else {
            continue;
        };
        let map_id = match repository::map_id_by_name(pool, map_name).await? {
            Some(id) => id,
            None => {
                repository::ensure_map(pool, map_ext_id, map_name, r.map_tier.unwrap_or(0)).await?
            }
        };
        let mode = r.mode.as_deref().unwrap_or("KZT");
        repository::upsert_player(pool, steam_id, &r.player.display_name).await?;
        repository::upsert_record(
            pool,
            &RecordUpsert {
                external_id: Some(&r.uuid),
                steam_id,
                map_id,
                mode,
                stage: r.stage,
                tickrate: if r.tickrate > 0 { r.tickrate } else { 128 },
                teleports: r.teleports,
                time_ms: secs_to_ms(r.time),
                points: r.points,
                source: "gokz_top",
                origin_server_id: r.server_id,
                server_name: r.server_name.as_deref(),
                server_group: None,
                wr_time_ms: r.wr_time.map(secs_to_ms),
                is_valid: r.is_valid.unwrap_or(true),
                created_at: parse_time(&r.created_on),
            },
        )
        .await?;
        n += 1;
    }

    // 3) 本站汇总分（同一 (地图, 关卡) 取最佳后求和，见 scoring）
    for mode in MODES {
        let best = best_points(pool, steam_id, mode).await?;
        repository::update_local_points(pool, steam_id, mode, scoring::total_points(best)).await?;
    }
    Ok(n)
}

/// 取某玩家某模式下每个 (地图, 关卡, 传送口径) 的最佳分数。
/// GOKZ 的 TP / PRO 是两张独立榜单、各自给分，故分组要带 `teleports > 0`，
/// 否则会把两张榜的分并成一条而少算。
async fn best_points(pool: &PgPool, steam_id: i64, mode: &str) -> Result<Vec<i64>, JobError> {
    let rows: Vec<(i64,)> = sqlx::query_as(
        r#"SELECT max(points)::BIGINT AS best
           FROM records
           WHERE player_steam_id = $1 AND mode = $2 AND is_valid
           GROUP BY map_id, stage, (teleports > 0)"#,
    )
    .bind(steam_id)
    .bind(mode)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

// ------------------------------ 服务器 / 封禁 ------------------------------

pub async fn run_servers(
    pool: &PgPool,
    src: &SourceClient,
    steam_key: &str,
) -> Result<usize, JobError> {
    // 服务器列表来源之一：Steam Web API（公开接口，需 key）。
    // 另一个来源是装了本站插件的服务器通过 `/api/v1/ingest/servers` 上报，读取时插件优先。
    let rows = Steam(src, steam_key).kz_servers().await?;
    // 先把上一轮 Steam 在线的置离线，本轮重新发现的再写回在线
    repository::mark_steam_servers_offline(pool).await?;
    let tiers = repository::map_tiers(pool).await?;
    let mut n = 0;
    for s in &rows {
        let Some((ip, port)) = s.ip_port() else {
            continue;
        };
        let map_tier = tiers.get(&s.map).copied().unwrap_or(0);
        let external_id = format!("steam:{ip}:{port}");
        repository::upsert_steam_server(
            pool,
            &ServerUpsert {
                external_id: &external_id,
                name: &s.name,
                map: &s.map,
                ip: &ip,
                port,
                player_count: s.players,
                max_players: s.max_players,
                country: None,
                city: None,
                region: None,
                group_name: None,
                group_custom_id: None,
                is_public: true,
                online: true,
                map_tier,
                player_names: &[],
                last_seen_at: None,
                source: "steam",
                appid: Some(s.appid),
                game_mode: Some(&s.gametype),
                steam_id: None,
            },
        )
        .await?;
        n += 1;
    }
    tracing::info!(servers = rows.len(), "Steam 服务器同步完成（KZ 服）");
    Ok(n)
}

pub async fn run_bans(pool: &PgPool, src: &SourceClient, limit: i64) -> Result<usize, JobError> {
    let page = GokzTop(src).bans(limit, 0).await?;
    let mut n = 0;
    for b in &page.data {
        let Some(steam_id) = b.player.steamid64.parse::<i64>().ok() else {
            continue;
        };
        repository::upsert_player(pool, steam_id, &b.player.display_name).await?;
        let reason = [
            b.ban_type.clone().unwrap_or_default(),
            b.notes.clone().unwrap_or_default(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
        let server = b
            .server
            .as_ref()
            .and_then(|s| s.name.clone())
            .unwrap_or_default();
        repository::upsert_ban(
            pool,
            &BanUpsert {
                external_id: &b.uuid,
                steam_id,
                name: &b.player.display_name,
                avatar: "",
                reason: &reason,
                server: &server,
                detail: b.stats.as_deref().unwrap_or_default(),
                banned_at: parse_time(&b.created_at),
                ban_type: b.ban_type.as_deref(),
                expires_at: b.expires_at.as_deref().map(parse_time),
                source: "gokz_top",
            },
        )
        .await?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_helpers() {
        assert_eq!(secs_to_ms(169.4221), 169_422);
        assert_eq!(secs_to_ms(0.0004), 0);
        let t = parse_time("2026-10-17T12:33:57Z");
        // 2026-10-17T12:33:57Z 的 Unix 秒
        assert_eq!(t.timestamp(), 1_792_240_437);
        // 非法时间退化为当前时间（不 panic）
        assert!(parse_time("not-a-time").timestamp() > 1_700_000_000);
    }
}
