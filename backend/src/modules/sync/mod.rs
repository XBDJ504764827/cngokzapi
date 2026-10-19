//! 外部数据同步（PROJECT §9.5）：按需回源 + 定时增量，全部写入本站 PG。
//!
//! - `queue`：`sync_jobs` 队列表的入队 / 认领 / 结算；
//! - `jobs`：各类任务的具体拉取与映射；
//! - `worker`：后台循环（定时入队 + 消费队列 + 启动种子）；
//! - `repository`：本模块唯一书写 SQL 的地方（幂等 upsert）。

pub mod jobs;
pub mod queue;
pub mod repository;
pub mod worker;

/// 任务类型常量（`sync_jobs.kind`）。
pub const KIND_MAPS: &str = "maps";
pub const KIND_LEADERBOARD: &str = "leaderboard";
pub const KIND_RECENT: &str = "recent";
pub const KIND_MAP: &str = "map";
pub const KIND_PLAYER: &str = "player";
/// 轻量玩家档案（只拉名称/头像/国家，1 次外部请求）。
pub const KIND_PLAYER_PROFILE: &str = "player_profile";
pub const KIND_SERVERS: &str = "servers";
pub const KIND_BANS: &str = "bans";
