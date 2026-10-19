//! 业务模块。层职责（固定）：
//! - `routes.rs`：只声明路径与 handler 映射，不含逻辑；
//! - `handlers.rs`：薄层，只做参数萃取（Query/Path/Json/State）与错误映射；
//! - `service.rs`：编排层，参数校验、缓存策略、分页，调用 repository；
//! - `repository.rs`：唯一可碰 DB / 外部数据的地方；
//! - `models.rs` / `dto.rs`：领域模型与传输对象，JSON 一律 snake_case，與前端类型一一对应。
//!
//! 非 CRUD 模块（见 PROJECT §9）：`sync`（按需回源 + 定时同步）、
//! `scoring`（计分，纯函数无 IO）；外部 HTTP 只出现在 `crate::sources`。

pub mod bans;
pub mod format;
pub mod health;
pub mod ingest;
pub mod leaderboard;
pub mod maps;
pub mod mode;
pub mod page;
pub mod players;
pub mod records;
pub mod scoring;
pub mod servers;
pub mod stats;
pub mod stream;
pub mod sync;
