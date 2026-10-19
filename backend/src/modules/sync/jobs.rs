//! 各类同步任务的实现：外部拉取 → 字段映射 → 幂等落库。
//!
//! 每个任务都是「可重入」的：重复执行只会更新已有行或跳过（PROJECT §9.6）。

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::repository::{self, BanUpsert, MapUpsert, RecordUpsert, ScoreUpsert, ServerUpsert};
use crate::{
    events::EventBus,
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
/// 全量导入时额外补头像/国籍的头部玩家数（按模式）。
const PROFILE_ENRICH_TOP: usize = 500;

/// 拉取某模式玩家榜前 `limit` 名（自动按 100 分页），写 players + player_scores。
pub async fn run_leaderboard(
    pool: &PgPool,
    src: &SourceClient,
    mode: &str,
    limit: i64,
    events: &EventBus,
) -> Result<usize, JobError> {
    let client = GokzTop(src);
    // 全量回扫（limit 很大）结束后清理掉榜行；快速刷新不做删除。
    let full = limit > 1_000_000;
    let started_at = Utc::now();
    let mut n = 0;
    let mut offset = 0i64;
    let mut enrich: Vec<String> = Vec::new();
    let mut profile_enrich: Vec<String> = Vec::new();
    while offset < limit {
        let want = LEADERBOARD_PAGE.min(limit - offset);
        let page = client.leaderboard(mode, want, offset).await?;
        if let Some(total) = page.count {
            tracing::info!(mode, upstream_total = total, "上游榜单总人数");
        }
        let got = page.data.len() as i64;
        let mut batch: Vec<repository::LeaderboardRow<'_>> = Vec::with_capacity(page.data.len());
        for row in &page.data {
            let Some(steam_id) = row.player.steamid64.parse::<i64>().ok() else {
                continue;
            };
            if enrich.len() < ENRICH_TOP {
                enrich.push(row.player.steamid64.clone());
            }
            if profile_enrich.len() < PROFILE_ENRICH_TOP {
                profile_enrich.push(row.player.steamid64.clone());
            }
            batch.push(repository::LeaderboardRow {
                steam_id,
                name: &row.player.display_name,
                points: row.points,
                rank: row.rank.or(row.global_rank),
                rating: row.rating,
                maps_completed: row.unique_map_finishes,
                wrs_nub: row.wrs_nub,
                wrs_pro: row.wrs_pro,
                records_900_plus: row.records_900_plus,
                records_800_plus: row.records_800_plus,
            });
        }
        n += repository::upsert_leaderboard_batch(pool, mode, &batch).await?;
        offset += got;
        if got < want || got == 0 {
            break;
        }
    }
    // 榜单前 N 名补全档案与 PB（6 小时内不重复入队，避免 10 分钟刷新时反复拉取）
    for steamid64 in enrich {
        super::queue::enqueue_if_stale(pool, super::KIND_PLAYER, &steamid64, 6 * 3600).await?;
    }
    // 头部玩家的头像/国籍补齐（24 小时内不重复），避免全量导入后大面积占位图
    for steamid64 in profile_enrich {
        super::queue::enqueue_if_stale(pool, super::KIND_PLAYER_PROFILE, &steamid64, 24 * 3600)
            .await?;
    }
    if full {
        let removed = repository::delete_stale_scores(pool, mode, started_at).await?;
        if removed > 0 {
            tracing::info!(mode, removed, "全量榜单已清理掉榜行");
        }
    }
    events.emit("leaderboard", "updated", None);
    Ok(n)
}

// ------------------------------ 增量同步：KZTimer 实时广播源 ------------------------------

/// KZTimer 拉取条数（接口不支持按模式过滤，拉回后本地筛选）。
const RECENT_FETCH: i64 = 1000;
/// KZTimer recent 的 record id 游标（只处理 id 增大的新记录）。
const CURSOR_KZTIMER_RECENT: &str = "kztimer_recent_id";

/// 拉取最新成绩：单任务处理三模式，按 record id 游标去重，只重算新增记录涉及的玩家。
pub async fn run_recent(
    pool: &PgPool,
    src: &SourceClient,
    events: &EventBus,
) -> Result<usize, JobError> {
    let rows = Kztimer(src).recent_records(RECENT_FETCH).await?;
    let cursor = repository::get_sync_state(pool, CURSOR_KZTIMER_RECENT)
        .await?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let max_id = rows.iter().map(|r| r.id).max().unwrap_or(cursor);

    let mut n = 0;
    let mut touched: Vec<(i64, String)> = Vec::new();
    for r in rows.iter().filter(|r| r.id > cursor) {
        // 接口不支持按模式过滤，这里本地筛选（PROJECT §9.2）
        let Some(mode) = kztimer::mode_name(&r.mode) else {
            continue;
        };
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
        let inserted = repository::upsert_record(
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
        if inserted {
            touched.push((steam_id, mode.to_string()));
            events.emit_record("created", map_id, steam_id);
            n += 1;
        }
    }
    if max_id > cursor {
        repository::set_sync_state(pool, CURSOR_KZTIMER_RECENT, &max_id.to_string()).await?;
    }
    refresh_local_points(pool, &touched).await?;
    Ok(n)
}

// ------------------------------ 增量同步：gokz.top 记录修改 / 软删除 ------------------------------

const RECORD_UPDATE_PAGE: i64 = 10_000;
const RECORD_UPDATE_MAX: i64 = 50_000;
const CURSOR_GOKZ_RECORDS: &str = "gokz_records_updated";

/// 拉取 `updated_since` 之后被修改 / 软删除的记录：
/// - 新成绩：正常落库；
/// - `is_valid=false`：把本地记录标为无效（地图榜 / 最近成绩立即消失）；
/// - 改分：更新 `points` 并重算受影响玩家。
pub async fn run_record_updates(
    pool: &PgPool,
    src: &SourceClient,
    events: &EventBus,
) -> Result<usize, JobError> {
    let since = repository::get_sync_state(pool, CURSOR_GOKZ_RECORDS)
        .await?
        .unwrap_or_else(|| iso_now_minus(600));
    let client = GokzTop(src);
    let since_time = parse_time(&since);
    let mut cursor = since_time;
    let mut offset = 0i64;
    let mut n = 0;
    let mut touched: Vec<(i64, String)> = Vec::new();

    loop {
        let page = client
            .records_updated_since(&since, RECORD_UPDATE_PAGE, offset)
            .await?;
        let got = page.data.len() as i64;
        for r in &page.data {
            if let Some(u) = r.updated_on.as_deref() {
                cursor = cursor.max(parse_time(u));
            }
            let (Some(map_name), Some(steam_id)) = (
                r.map_name.as_deref(),
                r.player.steamid64.parse::<i64>().ok(),
            ) else {
                continue;
            };
            let map_id = match repository::map_id_by_name(pool, map_name).await? {
                Some(id) => id,
                None => {
                    repository::ensure_map(
                        pool,
                        r.map_id.unwrap_or(0),
                        map_name,
                        r.map_tier.unwrap_or(0),
                    )
                    .await?
                }
            };
            let mode = r.mode.as_deref().unwrap_or("KZT");
            repository::upsert_player(pool, steam_id, &r.player.display_name).await?;
            let changed = repository::upsert_record_state(
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
            if changed {
                touched.push((steam_id, mode.to_string()));
                events.emit_record(
                    if r.is_valid.unwrap_or(true) {
                        "updated"
                    } else {
                        "invalidated"
                    },
                    map_id,
                    steam_id,
                );
                n += 1;
            }
        }
        offset += got;
        if got < RECORD_UPDATE_PAGE || offset >= RECORD_UPDATE_MAX {
            break;
        }
    }

    // 有更新时回退 60s 重叠，无更新时推进到最近窗口，避免反复扫同一段
    let next = if cursor > since_time {
        iso_of(cursor - chrono::Duration::seconds(60))
    } else {
        iso_now_minus(300)
    };
    repository::set_sync_state(pool, CURSOR_GOKZ_RECORDS, &next).await?;
    refresh_local_points(pool, &touched).await?;
    Ok(n)
}

/// 把时间格式化为 UTC RFC3339（秒级，`Z` 结尾），与 gokz.top 游标一致。
fn iso_of(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn iso_now_minus(secs: i64) -> String {
    iso_of(Utc::now() - chrono::Duration::seconds(secs))
}

/// 批量重算受影响玩家的本站汇总分（去重后逐个计算）。
async fn refresh_local_points(pool: &PgPool, touched: &[(i64, String)]) -> Result<(), JobError> {
    let mut seen = std::collections::HashSet::new();
    for (steam_id, mode) in touched {
        if !seen.insert((*steam_id, mode.clone())) {
            continue;
        }
        let best = best_points(pool, *steam_id, mode).await?;
        repository::update_local_points(pool, *steam_id, mode, scoring::total_points(best)).await?;
    }
    Ok(())
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

    // 0) 基础档案（名称 / 头像 / 国家），用于个人资料页的头像与国旗
    if let Some(p) = client.player_profile(steamid64).await? {
        repository::upsert_player_profile(
            pool,
            steam_id,
            &p.name,
            p.avatar_url().as_deref(),
            p.country.as_deref(),
        )
        .await?;
        n += 1;
    }

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

    // 2) 成绩：先按模式分页拉 PB（NUB/PRO 主关，覆盖全部完成地图），再拉最近成绩
    for mode in MODES {
        for rec_type in ["NUB", "PRO"] {
            let mut offset = 0i64;
            loop {
                let rows = client
                    .player_pbs(steamid64, mode, rec_type, false, PB_PAGE, offset)
                    .await?;
                let got = rows.len() as i64;
                n += upsert_top_records(pool, steam_id, &rows).await?;
                if got < PB_PAGE || offset + got >= PB_MAX {
                    break;
                }
                offset += got;
            }
        }
    }
    let page = client.player_records(steamid64, 1000).await?;
    n += upsert_top_records(pool, steam_id, &page.data).await?;

    // 3) 本站汇总分（同一 (地图, 关卡) 取最佳后求和，见 scoring）
    for mode in MODES {
        let best = best_points(pool, steam_id, mode).await?;
        repository::update_local_points(pool, steam_id, mode, scoring::total_points(best)).await?;
    }
    Ok(n)
}

/// PB 单页上限与单玩家最多翻页量（防上游异常）。
const PB_PAGE: i64 = 1000;
const PB_MAX: i64 = 20_000;

/// 把一批 gokz.top 成绩幂等写入本站（解析地图 id、补齐玩家档案）。
async fn upsert_top_records(
    pool: &PgPool,
    steam_id: i64,
    rows: &[TopRecordRow],
) -> Result<usize, JobError> {
    let mut n = 0;
    for r in rows {
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
    Ok(n)
}

/// 轻量玩家档案同步：只拉 `/players/{steamid64}`（名称 / 头像 / 国家），
/// 用于批量回填缺失头像与国旗，避免为每个玩家重跑完整档案。
pub async fn run_player_profile(
    pool: &PgPool,
    src: &SourceClient,
    steamid64: &str,
) -> Result<usize, JobError> {
    let Some(steam_id) = steamid64.parse::<i64>().ok() else {
        return Err(JobError::Db(sqlx::Error::RowNotFound));
    };
    let Some(p) = GokzTop(src).player_profile(steamid64).await? else {
        return Ok(0);
    };
    repository::upsert_player_profile(
        pool,
        steam_id,
        &p.name,
        p.avatar_url().as_deref(),
        p.country.as_deref(),
    )
    .await?;
    Ok(1)
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

    // Steam Web API 只有人数，没有昵称：用 gokz.top 的 live_status.players 补写。
    // 插件服以插件上报为准，本步骤只更新 source='steam' 的行。
    match GokzTop(src).servers().await {
        Ok(page) => {
            let mut named = 0;
            for s in &page.data {
                let Some(ls) = &s.live_status else {
                    continue;
                };
                let names: Vec<String> = ls
                    .players
                    .as_ref()
                    .map(|players| players.iter().filter_map(|p| p.name.clone()).collect())
                    .unwrap_or_default();
                if names.is_empty() {
                    continue;
                }
                if repository::update_server_player_names(pool, &s.ip, s.port, &names).await? > 0 {
                    named += 1;
                }
            }
            tracing::info!(named, "已补写 gokz.top 在线玩家昵称");
        }
        // 失败不影响服务器在线状态，只是暂时没有昵称
        Err(e) => tracing::warn!(error = %e, "gokz.top 在线玩家不可用，保留 Steam 人数"),
    }
    Ok(n)
}

const BAN_UPDATE_PAGE: i64 = 10_000;
const BAN_UPDATE_MAX: i64 = 50_000;
const CURSOR_GOKZ_BANS: &str = "gokz_bans_updated";

/// 增量拉取封禁，并在封禁发生时级联失效该玩家的成绩与榜单。
/// gokz.top 的封禁类型均为作弊类（hack/macro/ban_evasion/exploiting/boosting）。
pub async fn run_bans(
    pool: &PgPool,
    src: &SourceClient,
    events: &EventBus,
) -> Result<usize, JobError> {
    let since = repository::get_sync_state(pool, CURSOR_GOKZ_BANS)
        .await?
        .unwrap_or_else(|| iso_now_minus(3600));
    let client = GokzTop(src);
    let since_time = parse_time(&since);
    let mut cursor = since_time;
    let mut offset = 0i64;
    let mut n = 0;

    loop {
        let page = client
            .bans_updated_since(&since, BAN_UPDATE_PAGE, offset)
            .await?;
        let got = page.data.len() as i64;
        for b in &page.data {
            if let Some(u) = b.updated_at.as_deref() {
                cursor = cursor.max(parse_time(u));
            }
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
            // 级联：作弊封禁 → 成绩失效 + 移出排行榜（下一次解禁/上游恢复时自动重新计入）
            repository::invalidate_player_records(pool, steam_id).await?;
            events.emit("bans", "created", None);
            events.emit(&format!("player:{steam_id}"), "invalidated", Some(steam_id));
            events.emit("stats", "updated", None);
            n += 1;
        }
        offset += got;
        if got < BAN_UPDATE_PAGE || offset >= BAN_UPDATE_MAX {
            break;
        }
    }

    let next = if cursor > since_time {
        iso_of(cursor - chrono::Duration::seconds(60))
    } else {
        iso_now_minus(300)
    };
    repository::set_sync_state(pool, CURSOR_GOKZ_BANS, &next).await?;
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
