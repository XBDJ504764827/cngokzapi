use std::net::SocketAddr;

/// 服务配置，全部来自环境变量，方便本地与生产切换。
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    /// 静态网站目录，默认 `web`（workspace 根下）。
    /// 二进制位置变化时可用 WEB_DIR 覆盖，例如 WEB_DIR=/srv/cngokz/web。
    pub web_dir: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            web_dir: std::env::var("WEB_DIR").unwrap_or_else(|_| "web".to_string()),
        }
    }

    pub fn bind_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("HOST/PORT 不是合法的 SocketAddr")
    }
}
