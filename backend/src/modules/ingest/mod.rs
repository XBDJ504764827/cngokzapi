//! 上报接收端（PROJECT §9.6）：接收服务器插件/兼容客户端提交的成绩。
//!
//! - `POST /api/v1/ingest/records`：原生批量接口（自建 `cngokzapi` 插件用）；
//! - `POST /api/v2.0/records`：GlobalAPI 兼容层（字段与官方客户端一致）。
//!
//! 两者共用 `X-ApiKey`（服务器组 key）鉴权、幂等去重与本站计分汇总。

pub mod dto;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;
