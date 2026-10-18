//! 在线服务器列表（读本站 `servers`，由 `modules/sync` 定时同步）。
//! 安全约束：接口下发 ip/port 供「复制 IP」按钮使用，页面不渲染明文地址（PROJECT §2）。

pub mod category;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;
