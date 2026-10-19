//! 同步后台 worker：消费 `sync_jobs` + 定时入队 + 启动种子（PROJECT §9.5）。

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;

use tokio::sync::Semaphore;
use tokio::time::Instant;

use super::{jobs, queue};
use crate::state::AppState;

/// 每轮最多认领的任务数。
const BATCH: usize = 8;
/// 同时在跑的任务数上限（共享 `SourceClient` 的限速闸门，不会打爆上游）。
const MAX_INFLIGHT: usize = 8;
/// 轮询间隔。
const TICK: Duration = Duration::from_secs(2);

/// 定时任务周期（秒）：增量任务高频、全量任务低频。
const RECENT_EVERY: u64 = 10;
const RECORD_UPDATE_EVERY: u64 = 30;
const SERVERS_EVERY: u64 = 300;
const BANS_EVERY: u64 = 60;
const LEADERBOARD_EVERY: u64 = 600;
/// 全量榜单回扫周期（6 小时）；平时只刷新前 1000 名。
const LEADERBOARD_FULL_EVERY: u64 = 6 * 3600;
/// 快速刷新覆盖的名次数。
const LEADERBOARD_TOP: i64 = 1000;
const MAPS_EVERY: u64 = 86_400;

/// 入队：任意任务。
pub async fn enqueue(state: &AppState, kind: &str, key: &str) {
    if let Err(e) = queue::enqueue(&state.db, kind, key).await {
        tracing::warn!(kind, key, error = %e, "同步任务入队失败");
    }
}

/// 入队：某张地图某模式的整图同步（地图页首次访问时调用）。
pub async fn enqueue_map(state: &AppState, map_name: &str, mode: &str) {
    enqueue(state, super::KIND_MAP, &format!("{map_name}|{mode}")).await;
}

/// 入队：某玩家档案懒加载（个人资料页首次访问时调用）。
pub async fn enqueue_player(state: &AppState, steamid64: &str) {
    enqueue(state, super::KIND_PLAYER, steamid64).await;
}

/// 后台任务主循环：启动种子 → 定时入队 → 消费队列。
pub async fn run(state: AppState) {
    // 给 Web 服务一点启动时间，再开始对外请求
    tokio::time::sleep(Duration::from_secs(3)).await;
    if let Err(e) = seed(&state).await {
        tracing::warn!(error = %e, "同步启动种子失败（数据库不可用？）");
    }

    let mut last_recent = Instant::now() - Duration::from_secs(RECENT_EVERY);
    let mut last_record_updates = Instant::now() - Duration::from_secs(RECORD_UPDATE_EVERY);
    let mut last_servers = Instant::now() - Duration::from_secs(SERVERS_EVERY);
    let mut last_bans = Instant::now() - Duration::from_secs(BANS_EVERY);
    let mut last_leaderboard = Instant::now() - Duration::from_secs(LEADERBOARD_EVERY);
    // 全量榜单不在每次重启时重跑：按库里最后成功时间计算下次到期。
    let full_age = sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
        "SELECT min(last_success_at) FROM sync_jobs          WHERE kind = 'leaderboard' AND key LIKE '%:full'",
    )
    .fetch_one(&state.db)
    .await
    .ok()
    .flatten()
    .map(|t| (Utc::now() - t).num_seconds().max(0) as u64)
    .unwrap_or(LEADERBOARD_FULL_EVERY);
    let mut last_leaderboard_full =
        Instant::now() - Duration::from_secs(full_age.min(LEADERBOARD_FULL_EVERY));
    let mut last_maps = Instant::now() - Duration::from_secs(MAPS_EVERY);
    let inflight = Arc::new(Semaphore::new(MAX_INFLIGHT));

    loop {
        // 定时入队：先地图（tier 等基础数据），再其余全局任务，最后增量
        if last_maps.elapsed().as_secs() >= MAPS_EVERY {
            enqueue(&state, super::KIND_MAPS, "all").await;
            last_maps = Instant::now();
        }
        if last_leaderboard.elapsed().as_secs() >= LEADERBOARD_EVERY {
            for mode in jobs::MODES {
                enqueue(&state, super::KIND_LEADERBOARD, mode).await;
            }
            last_leaderboard = Instant::now();
        }
        if last_leaderboard_full.elapsed().as_secs() >= LEADERBOARD_FULL_EVERY {
            for mode in jobs::MODES {
                enqueue(&state, super::KIND_LEADERBOARD, &format!("{mode}:full")).await;
            }
            last_leaderboard_full = Instant::now();
        }
        if last_servers.elapsed().as_secs() >= SERVERS_EVERY {
            enqueue(&state, super::KIND_SERVERS, "all").await;
            last_servers = Instant::now();
        }
        if last_bans.elapsed().as_secs() >= BANS_EVERY {
            enqueue(&state, super::KIND_BANS, "all").await;
            last_bans = Instant::now();
        }
        if last_recent.elapsed().as_secs() >= RECENT_EVERY {
            enqueue(&state, super::KIND_RECENT, "all").await;
            last_recent = Instant::now();
        }
        if last_record_updates.elapsed().as_secs() >= RECORD_UPDATE_EVERY {
            enqueue(&state, super::KIND_RECORD_UPDATES, "all").await;
            last_record_updates = Instant::now();
        }

        // 消费队列：受 inflight 信号量限制，任务并发执行但共享上游限速
        for _ in 0..BATCH {
            let Ok(permit) = inflight.clone().try_acquire_owned() else {
                break;
            };
            match queue::claim(&state.db).await {
                Ok(Some((id, kind, key, attempts))) => {
                    let st = state.clone();
                    tokio::spawn(async move {
                        run_one(&st, id, &kind, &key, attempts).await;
                        drop(permit);
                    });
                }
                Ok(None) => break,
                Err(e) => {
                    tracing::warn!(error = %e, "认领同步任务失败（数据库不可用？）");
                    break;
                }
            }
        }
        tokio::time::sleep(TICK).await;
    }
}

/// 执行单个任务并结算。
async fn run_one(state: &AppState, id: i64, kind: &str, key: &str, attempts: i32) {
    let started = Instant::now();
    match execute(state, kind, key).await {
        Ok(n) => {
            tracing::info!(
                kind,
                key,
                rows = n,
                ms = started.elapsed().as_millis(),
                "同步任务完成"
            );
            if let Err(e) = queue::finish_ok(&state.db, id).await {
                tracing::warn!(error = %e, "同步任务结算失败");
            }
        }
        Err(e) => {
            let msg = e.to_string();
            tracing::warn!(kind, key, error = %msg, "同步任务失败");
            if let Err(e) = queue::finish_err(&state.db, id, attempts, &msg).await {
                tracing::warn!(error = %e, "同步任务失败结算写入失败");
            }
        }
    }
}

/// 按 kind 分发。
async fn execute(state: &AppState, kind: &str, key: &str) -> Result<usize, jobs::JobError> {
    let pool = &state.db;
    let src = &state.source;
    let events = &state.events;
    match kind {
        super::KIND_MAPS => jobs::run_maps(pool, src).await,
        super::KIND_LEADERBOARD => {
            let (mode, full) = match key.split_once(':') {
                Some((mode, "full")) => (mode, true),
                _ => (key, false),
            };
            let limit = if full { i64::MAX / 2 } else { LEADERBOARD_TOP };
            jobs::run_leaderboard(pool, src, mode, limit, events).await
        }
        super::KIND_RECENT => jobs::run_recent(pool, src, events).await,
        super::KIND_RECORD_UPDATES => jobs::run_record_updates(pool, src, events).await,
        super::KIND_SERVERS => jobs::run_servers(pool, src, &state.config.steam_api_key).await,
        super::KIND_BANS => jobs::run_bans(pool, src, events).await,
        super::KIND_MAP => match key.split_once('|') {
            Some((map_name, mode)) => jobs::run_map(pool, src, map_name, mode).await,
            None => Err(jobs::JobError::Db(sqlx::Error::Protocol(format!(
                "地图任务 key 非法：{key}"
            )))),
        },
        super::KIND_PLAYER => jobs::run_player(pool, src, key).await,
        super::KIND_PLAYER_PROFILE => jobs::run_player_profile(pool, src, key).await,
        other => Err(jobs::JobError::Db(sqlx::Error::Protocol(format!(
            "未知任务类型：{other}"
        )))),
    }
}

/// 启动种子：库里缺什么就补什么（只在空表时触发，避免每次启动重复拉取）。
async fn seed(state: &AppState) -> Result<(), sqlx::Error> {
    use super::repository::table_count;

    // 上次进程退出时 in-flight 的任务会停留在 running，重新排队
    let requeued = sqlx::query(
        "UPDATE sync_jobs SET status = 'pending', updated_at = now() WHERE status = 'running'",
    )
    .execute(&state.db)
    .await?
    .rows_affected();
    if requeued > 0 {
        tracing::info!(requeued, "重新排队上次未完成的同步任务");
    }

    if table_count(&state.db, "maps").await? == 0 {
        enqueue(state, super::KIND_MAPS, "all").await;
    }
    if table_count(&state.db, "player_scores").await? == 0 {
        for mode in jobs::MODES {
            enqueue(state, super::KIND_LEADERBOARD, &format!("{mode}:full")).await;
        }
    }
    if table_count(&state.db, "records").await? == 0 {
        enqueue(state, super::KIND_RECENT, "all").await;
    }
    if table_count(&state.db, "servers").await? == 0 {
        enqueue(state, super::KIND_SERVERS, "all").await;
    }
    if table_count(&state.db, "bans").await? == 0 {
        enqueue(state, super::KIND_BANS, "all").await;
    }
    Ok(())
}
