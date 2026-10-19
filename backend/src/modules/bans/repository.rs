//! 封禁名单数据源：读本站 `bans`（由 `modules/sync` 从上游导入，PROJECT §9.3）。

use sqlx::{FromRow, PgPool};

use super::models::BanEntry;

#[derive(FromRow)]
struct Row {
    player: String,
    steam_id: String,
    avatar: Option<String>,
    reason: String,
    banned_at: String,
    server: String,
    detail: String,
    total: i64,
}

/// 名称模糊 / SteamID64 子串搜索 + 分页；返回 `(条目, 命中总数)`。
pub async fn search(
    pool: &PgPool,
    query: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<BanEntry>, usize), sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(
        r#"SELECT b.name                                   AS player,
                  b.steam_id::TEXT                         AS steam_id,
                  COALESCE(NULLIF(b.avatar_url, ''), p.avatar_url, '') AS avatar,
                  b.reason                                 AS reason,
                  to_char(b.banned_at, 'YYYY-MM-DD')        AS banned_at,
                  b.server                                 AS server,
                  b.detail                                 AS detail,
                  count(*) OVER()                          AS total
           FROM bans b
           LEFT JOIN players p ON p.steam_id = b.steam_id
           WHERE ($1 = ''
                  OR b.name ILIKE '%' || $1 || '%'
                  OR b.steam_id::TEXT LIKE '%' || $1 || '%')
           ORDER BY b.banned_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(query)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = rows.first().map(|r| r.total).unwrap_or(0).max(0) as usize;
    let data = rows
        .into_iter()
        .map(|r| BanEntry {
            player: r.player,
            steam_id64: r.steam_id,
            avatar: r.avatar.unwrap_or_default(),
            reason: r.reason,
            banned_at: r.banned_at,
            server: r.server,
            detail: r.detail,
        })
        .collect();
    Ok((data, total))
}
