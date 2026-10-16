//! 基础设施：数据库、缓存等外部服务，只做连接与连通性，不写业务 SQL。

pub mod cache;
pub mod db;
