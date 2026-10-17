//! 上报接收端的数据访问（本模块 SQL 所在地）。
//!
//! 玩家与成绩的写入复用 `modules/sync/repository` 的幂等 upsert（自然键与本模块一致，
//! 避免两处维护同一套去重规则）；本文件只放接收端特有的查询与计分汇总。

use sqlx::{FromRow, PgPool};

/// 校验通过的 key 对应的服务器组。
#[derive(Debug, Clone, FromRow)]
pub struct KeyRow {
    pub group_id: i64,
    pub group_name: String,
    pub custom_id: Option<String>,
}

/// 用 `X-ApiKey` 查有效（未吊销）的服务器组。
pub async fn find_key(pool: &PgPool, api_key: &str) -> Result<Option<KeyRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT g.id AS group_id, g.name AS group_name, g.custom_id
           FROM server_keys k
           JOIN server_groups g ON g.id = k.group_id
           WHERE k.api_key = $1 AND k.revoked_at IS NULL"#,
    )
    .bind(api_key)
    .fetch_optional(pool)
    .await
}

/// 按上游地图 id 解析本站地图内部 id。
pub async fn map_id_by_external(
    pool: &PgPool,
    external_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT id FROM maps WHERE external_id = $1")
        .bind(external_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

/// 成绩的自然键（幂等判定与名次计算共用）。
#[derive(Debug, Clone, Copy)]
pub struct RecordKey<'a> {
    pub steam_id: i64,
    pub map_id: i64,
    pub mode: &'a str,
    pub stage: i32,
    pub tickrate: i32,
    pub teleports: i32,
    pub time_ms: i32,
}

/// 同一成绩是否已在近期入库（兼容层没有幂等键，用自然键 + 时间窗兜底重试）。
pub async fn recent_duplicate(
    pool: &PgPool,
    key: &RecordKey<'_>,
    window_secs: i64,
) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        r#"SELECT id FROM records
           WHERE player_steam_id = $1 AND map_id = $2 AND mode = $3 AND stage = $4
             AND tickrate = $5 AND teleports = $6 AND time_ms = $7
             AND created_at > now() - make_interval(secs => $8)
           LIMIT 1"#,
    )
    .bind(key.steam_id)
    .bind(key.map_id)
    .bind(key.mode)
    .bind(key.stage)
    .bind(key.tickrate)
    .bind(key.teleports)
    .bind(key.time_ms)
    .bind(window_secs as f64)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

/// 该成绩在其榜单中的名次（同地图 / 模式 / 关卡 / 传送口径，严格更快的条数 + 1）。
pub async fn place_in_board(
    pool: &PgPool,
    map_id: i64,
    mode: &str,
    stage: i32,
    use_teleports: bool,
    time_ms: i32,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"SELECT count(*) + 1
           FROM records
           WHERE map_id = $1 AND mode = $2 AND stage = $3
             AND (teleports > 0) = $4 AND is_valid AND time_ms < $5"#,
    )
    .bind(map_id)
    .bind(mode)
    .bind(stage)
    .bind(use_teleports)
    .bind(time_ms)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// 不分传送口径的名次（兼容层的 `top_overall_place`）。
pub async fn place_overall(
    pool: &PgPool,
    map_id: i64,
    mode: &str,
    stage: i32,
    time_ms: i32,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"SELECT count(*) + 1
           FROM records
           WHERE map_id = $1 AND mode = $2 AND stage = $3 AND is_valid AND time_ms < $4"#,
    )
    .bind(map_id)
    .bind(mode)
    .bind(stage)
    .bind(time_ms)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// 重算某玩家某模式的本地汇总分（口径见 `modules/scoring` 与 PROJECT §9.4：
/// 按 (地图, 关卡, 传送口径) 取最佳后求和）。返回新的汇总值。
pub async fn recompute_local_points(
    pool: &PgPool,
    steam_id: i64,
    mode: &str,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"WITH best AS (
               SELECT max(points)::BIGINT AS p
               FROM records
               WHERE player_steam_id = $1 AND mode = $2 AND is_valid
               GROUP BY map_id, stage, (teleports > 0)
           )
           -- sum(bigint) 在 PG 里返回 NUMERIC，必须显式转回 BIGINT
           SELECT COALESCE(sum(p), 0)::BIGINT FROM best"#,
    )
    .bind(steam_id)
    .bind(mode)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
