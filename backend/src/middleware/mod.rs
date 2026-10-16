//! 中间件：认证、日志、CORS。请求日志由 tower-http `TraceLayer` 提供（见 `app.rs`）。

pub mod cors;
