//! 服务器数据源：读本站 `servers`。

use sqlx::{FromRow, PgPool};

use super::models::ServerEntry;

#[derive(FromRow)]
struct Row {
    id: i64,
    name: String,
    map: Option<String>,
    ip: String,
    port: i32,
    player_count: i32,
    max_players: i32,
    country: Option<String>,
    city: Option<String>,
    region: Option<String>,
    group_name: Option<String>,
    online: bool,
    map_tier: i16,
    player_names: Vec<String>,
}

/// 在线且公开的服务器，按在线人数倒序。
pub async fn list_online(pool: &PgPool) -> Result<Vec<ServerEntry>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(
        r#"SELECT s.id, s.name, s.map, s.ip, s.port, s.player_count, s.max_players,
                  s.country, s.city, s.region, s.group_name, s.online,
                  -- 优先用 maps 的当前难度，避免同步先后导致 0
                  COALESCE(NULLIF(s.map_tier, 0), m.tier_kzt, m.tier, 0)::SMALLINT AS map_tier,
                  s.player_names
           FROM servers s
           LEFT JOIN maps m ON m.name = s.map
           WHERE s.online AND s.is_public
           ORDER BY s.player_count DESC, s.name ASC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ServerEntry {
            id: r.id,
            name: r.name,
            map: r.map.unwrap_or_default(),
            ip: r.ip,
            port: r.port,
            player_count: r.player_count,
            max_players: r.max_players,
            country: r.country,
            city: r.city,
            region: r.region,
            group_name: r.group_name,
            online: r.online,
            map_tier: r.map_tier,
            player_names: r.player_names,
        })
        .collect())
}
