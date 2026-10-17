//! 最新记录数据源：读本站 `records`（按创建时间倒序，主关 stage = 0）。

use sqlx::{FromRow, PgPool};

use super::models::RecordEntry;
use crate::modules::format::format_duration_ms;

#[derive(FromRow)]
struct Row {
    player: String,
    steam_id: String,
    map: String,
    mode: String,
    time_ms: i32,
    points: i32,
    date: String,
}

const SQL: &str = r#"
SELECT p.name                              AS player,
       p.steam_id::TEXT                     AS steam_id,
       m.name                              AS map,
       r.mode                              AS mode,
       r.time_ms                           AS time_ms,
       r.points                            AS points,
       to_char(r.created_at, 'YYYY-MM-DD')  AS date
FROM records r
JOIN players p ON p.steam_id = r.player_steam_id
JOIN maps m ON m.id = r.map_id
WHERE r.mode = $1 AND r.stage = 0
ORDER BY r.created_at DESC
LIMIT $2
"#;

/// 某模式最新记录。
pub async fn latest_by_mode(
    pool: &PgPool,
    mode: &str,
    limit: i64,
) -> Result<Vec<RecordEntry>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(SQL)
        .bind(mode)
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| RecordEntry {
            player: r.player,
            steam_id64: r.steam_id,
            map: r.map,
            mode: r.mode,
            time: format_duration_ms(r.time_ms),
            points: i64::from(r.points),
            date: r.date,
        })
        .collect())
}
