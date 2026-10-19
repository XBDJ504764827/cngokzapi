//! cngokz-backend 库目标：把各模块暴露给 `main.rs`、`src/bin/*` 与集成测试。
//! 分层与职责见 PROJECT.md §3.3。

pub mod app;
pub mod config;
pub mod errors;
pub mod events;
pub mod infrastructure;
pub mod middleware;
pub mod modules;
pub mod sources;
pub mod state;
