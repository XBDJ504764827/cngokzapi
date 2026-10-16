//! 业务模块。层职责（固定）：
//! - `routes.rs`：只声明路径与 handler 映射，不含逻辑；
//! - `handlers.rs`：薄层，只做参数萃取（Query/Path/Json/State）与错误映射；
//! - `service.rs`：编排层，参数校验、缓存策略、分页，调用 repository；
//! - `repository.rs`：唯一可碰 DB / 外部数据的地方，占位阶段返回模拟数据；
//! - `models.rs` / `dto.rs`：领域模型与传输对象，JSON 一律 snake_case，與前端类型一一对应。

pub mod bans;
pub mod health;
pub mod leaderboard;
pub mod records;

mod mock;
pub mod page;
