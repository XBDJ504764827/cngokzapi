//! 服务器组与上报 key 管理（PROJECT §9.6）。
//!
//! 用法（先确保 `.env` 里的 `DATABASE_URL` 可用）：
//!
//! ```bash
//! cargo run --bin ingest-key -- create-group "CNGOKZ" cngokz
//! cargo run --bin ingest-key -- issue-key 1 "主服务器组"
//! cargo run --bin ingest-key -- list
//! cargo run --bin ingest-key -- revoke <api_key>
//! cargo run --bin ingest-key -- delete-record <record_uuid>
//! cargo run --bin ingest-key -- jobs            # 查看同步队列状态
//! cargo run --bin ingest-key -- requeue <kind> <key>   # 立即重跑某个同步任务
//! ```
//!
//! 拿到 key 后，插件/客户端以 `X-ApiKey: <key>` 调用
//! `POST /api/v1/ingest/records`（原生）或 `POST /api/v2.0/records`（GlobalAPI 兼容）。
//! `delete-record` 用于删除误报/测试成绩（面向管理员的管理接口见 PROJECT §10 下一步）。

use cngokz_backend::{config::AppConfig, infrastructure::db};
use rand::Rng;
use sqlx::{FromRow, PgPool};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    let pool = db::connect_lazy(&AppConfig::from_env().database_url);
    let result = match args[0].as_str() {
        "create-group" => create_group(&pool, &args[1..]).await,
        "issue-key" => issue_key(&pool, &args[1..]).await,
        "list" => list(&pool).await,
        "revoke" => revoke(&pool, &args[1..]).await,
        "delete-record" => delete_record(&pool, &args[1..]).await,
        "jobs" => list_jobs(&pool).await,
        "requeue" => requeue(&pool, &args[1..]).await,
        other => {
            eprintln!("未知命令：{other}");
            usage();
            std::process::exit(2);
        }
    };
    if let Err(e) = result {
        eprintln!("执行失败：{e}");
        std::process::exit(1);
    }
}

fn usage() {
    eprintln!(
        "用法：\n  \
         ingest-key create-group <名称> [custom_id]\n  \
         ingest-key issue-key <group_id> [标签]\n  \
         ingest-key list\n  \
         ingest-key revoke <api_key>\n  \
         ingest-key delete-record <record_uuid>\n  \
         ingest-key jobs\n  \
         ingest-key requeue <kind> <key>"
    );
}

/// 生成 32 位十六进制随机 key。
fn new_api_key() -> String {
    let bytes: [u8; 16] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

async fn create_group(pool: &PgPool, args: &[String]) -> Result<(), sqlx::Error> {
    let Some(name) = args.first() else {
        eprintln!("缺少名称");
        std::process::exit(2);
    };
    let custom_id = args.get(1).map(String::as_str);
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO server_groups (name, custom_id) VALUES ($1, $2)
           ON CONFLICT (custom_id) DO UPDATE SET name = EXCLUDED.name
           RETURNING id"#,
    )
    .bind(name)
    .bind(custom_id)
    .fetch_one(pool)
    .await?;
    println!(
        "服务器组已就绪：id={} name={} custom_id={}",
        row.0,
        name,
        custom_id.unwrap_or("-")
    );
    println!("下一步：ingest-key issue-key {} \"<标签>\"", row.0);
    Ok(())
}

async fn issue_key(pool: &PgPool, args: &[String]) -> Result<(), sqlx::Error> {
    let Some(group_id) = args.first().and_then(|s| s.parse::<i64>().ok()) else {
        eprintln!("缺少合法的 group_id");
        std::process::exit(2);
    };
    let label = args.get(1).map(String::as_str).unwrap_or("");
    let key = new_api_key();
    sqlx::query("INSERT INTO server_keys (group_id, api_key, label) VALUES ($1, $2, $3)")
        .bind(group_id)
        .bind(&key)
        .bind(label)
        .execute(pool)
        .await?;
    println!("已签发服务器组 key（只显示这一次，请妥善保存）：");
    println!("X-ApiKey: {key}");
    println!("组 id={group_id} 标签={label}");
    Ok(())
}

#[derive(FromRow)]
struct KeyListRow {
    group_id: i64,
    group_name: String,
    custom_id: Option<String>,
    api_key: String,
    label: String,
    revoked: bool,
}

async fn list(pool: &PgPool) -> Result<(), sqlx::Error> {
    let rows: Vec<KeyListRow> = sqlx::query_as(
        r#"SELECT k.group_id, g.name AS group_name, g.custom_id, k.api_key, k.label,
                  (k.revoked_at IS NOT NULL) AS revoked
           FROM server_keys k
           JOIN server_groups g ON g.id = k.group_id
           ORDER BY k.group_id, k.id"#,
    )
    .fetch_all(pool)
    .await?;
    if rows.is_empty() {
        println!("还没有任何服务器组 key，先 create-group 再 issue-key。");
        return Ok(());
    }
    for r in rows {
        // 列表只显示前缀，避免整表 key 出现在日志/终端回滚里
        let prefix: String = r.api_key.chars().take(8).collect();
        println!(
            "组 {:>3} {:<20} custom_id={:<12} key={}… label={} {}",
            r.group_id,
            r.group_name,
            r.custom_id.unwrap_or("-".to_string()),
            prefix,
            r.label,
            if r.revoked { "[已吊销]" } else { "" }
        );
    }
    Ok(())
}

async fn revoke(pool: &PgPool, args: &[String]) -> Result<(), sqlx::Error> {
    let Some(key) = args.first() else {
        eprintln!("缺少 api_key");
        std::process::exit(2);
    };
    let n = sqlx::query(
        "UPDATE server_keys SET revoked_at = now() WHERE api_key = $1 AND revoked_at IS NULL",
    )
    .bind(key)
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 {
        println!("没有找到该 key（或已吊销）。");
    } else {
        println!("已吊销 1 个 key。");
    }
    Ok(())
}

/// 删除一条已入库成绩（按上报幂等键 `external_id`）。
async fn delete_record(pool: &PgPool, args: &[String]) -> Result<(), sqlx::Error> {
    let Some(uuid) = args.first() else {
        eprintln!("缺少 record_uuid");
        std::process::exit(2);
    };
    let n = sqlx::query("DELETE FROM records WHERE external_id = $1")
        .bind(uuid)
        .execute(pool)
        .await?
        .rows_affected();
    println!("已删除 {n} 条记录（record_uuid={uuid}）");
    Ok(())
}

/// 查看同步队列（排查同步卡住/失败）。
async fn list_jobs(pool: &PgPool) -> Result<(), sqlx::Error> {
    #[derive(FromRow)]
    struct JobRow {
        kind: String,
        key: String,
        status: String,
        attempts: i32,
        next_run_at: chrono::DateTime<chrono::Utc>,
        last_error: Option<String>,
    }
    let rows: Vec<JobRow> = sqlx::query_as(
        r#"SELECT kind, key, status, attempts, next_run_at, last_error
           FROM sync_jobs
           ORDER BY (status = 'running') DESC, next_run_at ASC"#,
    )
    .fetch_all(pool)
    .await?;
    let mut counts = std::collections::BTreeMap::new();
    for r in &rows {
        *counts.entry(r.status.clone()).or_insert(0) += 1;
    }
    let mut by_kind = std::collections::BTreeMap::new();
    for r in &rows {
        *by_kind.entry(r.kind.as_str()).or_insert(0usize) += 1;
    }
    println!("队列统计：{counts:?}");
    println!("按类型：{by_kind:?}");
    let show = |r: &JobRow| {
        println!(
            "  {:<12} {:<26} {:<8} attempts={} next={} {}",
            r.kind,
            r.key,
            r.status,
            r.attempts,
            r.next_run_at.format("%H:%M:%S"),
            r.last_error.as_deref().unwrap_or("")
        );
    };
    println!("== 全局任务 ==");
    for r in rows.iter().filter(|r| r.kind != "player") {
        show(r);
    }
    println!("== 玩家任务（未完成，最多 15 条）==");
    for r in rows
        .iter()
        .filter(|r| r.kind == "player" && r.status != "done")
        .take(15)
    {
        show(r);
    }
    Ok(())
}

/// 立即重跑某个同步任务（把 next_run_at 提前并置为 pending）。
async fn requeue(pool: &PgPool, args: &[String]) -> Result<(), sqlx::Error> {
    let (Some(kind), Some(key)) = (args.first(), args.get(1)) else {
        eprintln!(
            "用法：requeue <kind> <key>（kind 如 maps/servers/bans/recent/leaderboard/map/player）"
        );
        std::process::exit(2);
    };
    let n = sqlx::query(
        r#"UPDATE sync_jobs
           SET status = 'pending', attempts = 0, next_run_at = now(), last_error = NULL, updated_at = now()
           WHERE kind = $1 AND key = $2"#,
    )
    .bind(kind)
    .bind(key)
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 {
        println!("没有该任务（{kind} / {key}）；若是首次，请等后台定时入队。");
    } else {
        println!("已重新排队：{kind} / {key}");
    }
    Ok(())
}
