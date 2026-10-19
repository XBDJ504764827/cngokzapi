//! 首页统计数据源：各表计数（本站 PG）。

use sqlx::{FromRow, PgPool};

use super::models::HomeStats;

#[derive(FromRow)]
struct Counts {
    players: i64,
    maps: i64,
    records: i64,
    servers_online: i64,
    players_online: Option<i64>,
    bans: i64,
}

/// 一次查询取回全部计数，避免多轮往返。
pub async fn home(pool: &PgPool) -> Result<HomeStats, sqlx::Error> {
    let c: Counts = sqlx::query_as(
        r#"SELECT (SELECT count(*) FROM players)                              AS players,
                  (SELECT count(*) FROM maps)                                 AS maps,
                  (SELECT count(*) FROM records)                              AS records,
                  (SELECT count(*) FROM servers
                    WHERE online AND is_public AND COALESCE(appid, 4465480) <> 730) AS servers_online,
                  (SELECT sum(player_count) FROM servers
                    WHERE online AND is_public AND COALESCE(appid, 4465480) <> 730) AS players_online,
                  (SELECT count(*) FROM bans)                                 AS bans"#,
    )
    .fetch_one(pool)
    .await?;

    Ok(HomeStats {
        players: c.players,
        maps: c.maps,
        records: c.records,
        servers_online: c.servers_online,
        players_online: c.players_online.unwrap_or(0),
        bans: c.bans,
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}
