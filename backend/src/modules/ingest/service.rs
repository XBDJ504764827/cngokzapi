//! 编排层：鉴权 → 校验 → 幂等落库 → 名次与本地汇总（PROJECT §9.6）。

use chrono::{DateTime, Utc};

use super::{
    dto::{CompatRecord, IngestItem},
    models::{CompatResponse, IngestResult},
    repository,
};
use crate::{
    errors::AppError,
    modules::sync::repository::{self as sync_repo, RecordUpsert},
    state::AppState,
};

/// 单条成绩的处理错误：`Reject` 只影响该条（返回 `rejected`），
/// `Db` 视为系统故障（整批返回 500，客户端应重试）。
enum ItemError {
    Reject(String),
    Db(sqlx::Error),
}

impl From<sqlx::Error> for ItemError {
    fn from(e: sqlx::Error) -> Self {
        ItemError::Db(e)
    }
}

impl From<AppError> for ItemError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::BadRequest(m) => ItemError::Reject(m),
            other => ItemError::Reject(other.to_string()),
        }
    }
}

/// 单批上限：避免一次请求打满连接与磁盘。
pub const MAX_BATCH: usize = 200;
/// 兼容层重复判定时间窗（秒）：官方客户端没有幂等键，同成绩在窗口内视为重试。
const COMPAT_DUP_WINDOW_SECS: i64 = 600;
/// 上传来源标记（PROJECT §9.3 数据通路）。
const SOURCE_PLUGIN: &str = "plugin";

/// 鉴权结果。
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub group_id: i64,
    pub group_name: String,
    pub custom_id: Option<String>,
}

/// 用 `X-ApiKey` 鉴权（服务器组 key，PROJECT §9.6）。
pub async fn authenticate(state: &AppState, key: Option<&str>) -> Result<AuthContext, AppError> {
    let key = key.map(str::trim).filter(|k| !k.is_empty());
    let Some(key) = key else {
        return Err(AppError::Unauthorized);
    };
    match repository::find_key(&state.db, key).await? {
        Some(row) => Ok(AuthContext {
            group_id: row.group_id,
            group_name: row.group_name,
            custom_id: row.custom_id,
        }),
        None => Err(AppError::Unauthorized),
    }
}

/// SteamID64：17 位纯数字。
pub fn parse_steam_id64(raw: &str) -> Result<i64, String> {
    let raw = raw.trim();
    if raw.len() != 17 || !raw.chars().all(|c| c.is_ascii_digit()) {
        return Err("steam_id64 应为 17 位数字".to_string());
    }
    raw.parse::<i64>()
        .map_err(|_| "steam_id64 非法".to_string())
}

/// `STEAM_1:0:182656610` → SteamID64（`base + authid * 2 + server`）。
pub fn steam32_to_steam64(raw: &str) -> Result<i64, String> {
    const BASE: i64 = 76_561_197_960_265_728;
    let raw = raw.trim();
    let mut parts = raw.split(':');
    let universe = parts.next().unwrap_or_default();
    let server = parts.next().unwrap_or_default();
    let auth_id = parts.next().unwrap_or_default();
    if !universe.starts_with("STEAM_") || parts.next().is_some() {
        return Err(format!("steam_id 格式应为 STEAM_x:y:z，收到 {raw}"));
    }
    let server: i64 = server
        .parse()
        .map_err(|_| format!("steam_id 的 y 段非法：{raw}"))?;
    let auth_id: i64 = auth_id
        .parse()
        .map_err(|_| format!("steam_id 的 z 段非法：{raw}"))?;
    if !(0..=1).contains(&server) || auth_id < 0 {
        return Err(format!("steam_id 取值非法：{raw}"));
    }
    Ok(BASE + auth_id * 2 + server)
}

/// 上游 GlobalMode（200/201/202）或模式名 → 本站模式。
pub fn normalize_mode(value: &serde_json::Value) -> Result<&'static str, String> {
    if let Some(n) = value.as_i64() {
        return match n {
            200 => Ok("KZT"),
            201 => Ok("SKZ"),
            202 => Ok("VNL"),
            other => Err(format!("mode 非法：{other}（应为 200/201/202）")),
        };
    }
    let s = value
        .as_str()
        .ok_or_else(|| "mode 应为数字或字符串".to_string())?;
    let up = s.trim().to_ascii_uppercase();
    match up.as_str() {
        "KZT" | "KZ_TIMER" => Ok("KZT"),
        "SKZ" | "KZ_SIMPLE" => Ok("SKZ"),
        "VNL" | "KZ_VANILLA" => Ok("VNL"),
        other => Err(format!("mode 非法：{other}")),
    }
}

/// RFC3339 → UTC；缺失或非法则取当前时间。
fn parse_time_or_now(raw: Option<&str>) -> DateTime<Utc> {
    raw.and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(Utc::now)
}

/// 秒（浮点）→ 毫秒整数。
fn secs_to_ms(v: f64) -> i32 {
    (v * 1000.0).round() as i32
}

/// 原生批量上报。
pub async fn ingest_batch(
    state: &AppState,
    auth: &AuthContext,
    items: &[IngestItem],
) -> Result<Vec<IngestResult>, AppError> {
    let mut results = Vec::with_capacity(items.len());
    let mut touched: Vec<(i64, &'static str)> = Vec::new();
    for item in items {
        match ingest_one(state, auth, item).await {
            Ok(result) => {
                if result.status == "accepted" {
                    if let (Ok(steam_id), Ok(m)) = (
                        parse_steam_id64(&item.steam_id64),
                        normalize_mode(&serde_json::json!(item.mode)),
                    ) {
                        touched.push((steam_id, m));
                    }
                }
                results.push(result);
            }
            Err(ItemError::Reject(reason)) => {
                results.push(IngestResult::rejected(item.record_uuid.clone(), reason))
            }
            // 数据库故障不是"这条成绩不合格"，整批失败让客户端稍后重试
            Err(ItemError::Db(e)) => return Err(e.into()),
        }
    }
    refresh_local_points(state, &mut touched).await?;
    Ok(results)
}

async fn ingest_one(
    state: &AppState,
    auth: &AuthContext,
    item: &IngestItem,
) -> Result<IngestResult, ItemError> {
    let steam_id = parse_steam_id64(&item.steam_id64).map_err(ItemError::Reject)?;
    let m = normalize_mode(&serde_json::json!(item.mode)).map_err(ItemError::Reject)?;
    if item.stage < 0 {
        return Err(ItemError::Reject("stage 不能为负".to_string()));
    }
    if !(64..=128).contains(&item.tickrate) {
        return Err(ItemError::Reject("tickrate 仅支持 64 / 128".to_string()));
    }
    if item.teleports < 0 {
        return Err(ItemError::Reject("teleports 不能为负".to_string()));
    }
    if item.time_ms <= 0 {
        return Err(ItemError::Reject(
            "time_ms 必须为正整数（毫秒）".to_string(),
        ));
    }

    // 地图：优先外部 id，其次名称
    let map_id = match (item.map_id, item.map_name.as_deref()) {
        (Some(ext), _) => repository::map_id_by_external(&state.db, ext).await?,
        (None, Some(name)) => sync_repo::map_id_by_name(&state.db, name).await?,
        (None, None) => return Err(ItemError::Reject("必须提供 map_name 或 map_id".to_string())),
    };
    let Some(map_id) = map_id else {
        return Err(ItemError::Reject(
            "地图不存在（请先在 /maps 中确认）".to_string(),
        ));
    };

    let name = item.player_name.clone().unwrap_or_default();
    sync_repo::upsert_player(&state.db, steam_id, &name).await?;

    let inserted = sync_repo::upsert_record(
        &state.db,
        &RecordUpsert {
            external_id: item.record_uuid.as_deref(),
            steam_id,
            map_id,
            mode: m,
            stage: item.stage,
            tickrate: item.tickrate,
            teleports: item.teleports,
            time_ms: item.time_ms,
            // 单条成绩分：本站 GOKZ 公式尚未落地（PROJECT §9.4 第 1 层），先记 0
            points: 0,
            source: SOURCE_PLUGIN,
            origin_server_id: item.server_id,
            server_name: None,
            server_group: auth.custom_id.as_deref(),
            wr_time_ms: None,
            is_valid: true,
            created_at: parse_time_or_now(item.created_on.as_deref()),
        },
    )
    .await?;

    if !inserted {
        return Ok(IngestResult::duplicate(item.record_uuid.clone()));
    }
    let place = repository::place_in_board(
        &state.db,
        map_id,
        m,
        item.stage,
        item.teleports > 0,
        item.time_ms,
    )
    .await?;
    Ok(IngestResult::accepted(item.record_uuid.clone(), place))
}

/// GlobalAPI 兼容上报（字段与官方客户端一致）。
pub async fn ingest_compat(
    state: &AppState,
    auth: &AuthContext,
    item: &CompatRecord,
) -> Result<CompatResponse, AppError> {
    let steam_id = steam32_to_steam64(&item.steam_id)
        .map_err(|e| AppError::BadRequest(format!("steam_id 非法：{e}")))?;
    let m = normalize_mode(&item.mode).map_err(AppError::BadRequest)?;
    if item.tickrate != 64 && item.tickrate != 128 {
        return Err(AppError::BadRequest("tickrate 仅支持 64 / 128".to_string()));
    }
    let time_ms = secs_to_ms(item.time);
    if time_ms <= 0 {
        return Err(AppError::BadRequest("time 必须为正数（秒）".to_string()));
    }
    let map_id = repository::map_id_by_external(&state.db, item.map_id)
        .await?
        .ok_or_else(|| AppError::BadRequest(format!("地图不存在：map_id={}", item.map_id)))?;

    // 官方客户端没有幂等键：同成绩在时间窗内重复提交视为重试
    let key = repository::RecordKey {
        steam_id,
        map_id,
        mode: m,
        stage: item.stage,
        tickrate: item.tickrate,
        teleports: item.teleports,
        time_ms,
    };
    if repository::recent_duplicate(&state.db, &key, COMPAT_DUP_WINDOW_SECS).await? {
        let place = repository::place_in_board(
            &state.db,
            map_id,
            m,
            item.stage,
            item.teleports > 0,
            time_ms,
        )
        .await?;
        let overall = repository::place_overall(&state.db, map_id, m, item.stage, time_ms).await?;
        return Ok(CompatResponse {
            place,
            top_place: place,
            top_overall_place: overall,
        });
    }

    sync_repo::upsert_player(&state.db, steam_id, "").await?;
    sync_repo::upsert_record(
        &state.db,
        &RecordUpsert {
            external_id: None,
            steam_id,
            map_id,
            mode: m,
            stage: item.stage,
            tickrate: item.tickrate,
            teleports: item.teleports,
            time_ms,
            points: 0,
            source: SOURCE_PLUGIN,
            origin_server_id: None,
            server_name: None,
            server_group: auth.custom_id.as_deref(),
            wr_time_ms: None,
            is_valid: true,
            created_at: Utc::now(),
        },
    )
    .await?;

    let place = repository::place_in_board(
        &state.db,
        map_id,
        m,
        item.stage,
        item.teleports > 0,
        time_ms,
    )
    .await?;
    let overall = repository::place_overall(&state.db, map_id, m, item.stage, time_ms).await?;
    let mut touched = vec![(steam_id, m)];
    refresh_local_points(state, &mut touched).await?;

    Ok(CompatResponse {
        place,
        top_place: place,
        top_overall_place: overall,
    })
}

/// 受理后重算受影响玩家的本地汇总分（去重后逐个重算）。
async fn refresh_local_points(
    state: &AppState,
    touched: &mut Vec<(i64, &'static str)>,
) -> Result<(), AppError> {
    touched.sort_unstable();
    touched.dedup();
    for (steam_id, m) in touched.iter() {
        let total = repository::recompute_local_points(&state.db, *steam_id, m).await?;
        sync_repo::update_local_points(&state.db, *steam_id, m, total).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_steam_id64() {
        assert_eq!(
            parse_steam_id64("76561198325578948").unwrap(),
            76_561_198_325_578_948
        );
        assert!(parse_steam_id64("123").is_err());
        assert!(parse_steam_id64("7656119832557894a").is_err());
    }

    #[test]
    fn converts_steam32() {
        // 与上游返回的一一对应关系（见 CHANGELOG 2026-10-17 探测记录）
        assert_eq!(
            steam32_to_steam64("STEAM_1:0:182656610").unwrap(),
            76_561_198_325_578_948
        );
        assert_eq!(
            steam32_to_steam64("STEAM_1:1:168575119").unwrap(),
            76_561_198_297_415_967
        );
        assert!(steam32_to_steam64("76561198325578948").is_err());
        assert!(steam32_to_steam64("STEAM_1:2:1").is_err());
    }

    #[test]
    fn normalizes_modes() {
        assert_eq!(normalize_mode(&json!(200)).unwrap(), "KZT");
        assert_eq!(normalize_mode(&json!(201)).unwrap(), "SKZ");
        assert_eq!(normalize_mode(&json!(202)).unwrap(), "VNL");
        assert_eq!(normalize_mode(&json!("kz_vanilla")).unwrap(), "VNL");
        assert!(normalize_mode(&json!(999)).is_err());
        assert!(normalize_mode(&json!(true)).is_err());
    }

    #[test]
    fn converts_seconds_to_ms() {
        assert_eq!(secs_to_ms(169.4221), 169_422);
        assert_eq!(secs_to_ms(0.0004), 0);
    }
}
