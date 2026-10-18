//! 配置管理：全部来自环境变量，详见根目录 `.env.example`。
use std::net::SocketAddr;

/// 服务配置，全部来自环境变量，方便本地与生产切换。
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    /// 前端构建产物目录，默认 `frontend/dist`（workspace 根下相对路径）。
    /// 未构建前端时 API 照常可用，静态页返回缺失提示，见 `app.rs`。
    pub web_dir: String,
    /// PostgreSQL 连接串。懒连接，无库也可启动，数据访问失败时接口返回 503。
    pub database_url: String,
    /// Redis 连接串。Client 不建连，无库也可启动。
    pub redis_url: String,
    /// Steam Web API key（IGameServersService 查询 KZ 服务器列表）。空则不启用 Steam 源。
    pub steam_api_key: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            // 默认监听全网段，局域网可经 http://192.168.0.101:PORT 访问；
            // 如只想本机访问，可设 HOST=127.0.0.1；如只想固定网卡，可设 HOST=192.168.0.101。
            host: env("HOST", "0.0.0.0"),
            port: env("PORT", "3000").parse().unwrap_or(3000),
            web_dir: env("WEB_DIR", "frontend/dist"),
            database_url: env(
                "DATABASE_URL",
                "postgres://cngokz:cngokz@127.0.0.1:5432/cngokz",
            ),
            redis_url: env("REDIS_URL", "redis://127.0.0.1:6379"),
            steam_api_key: env("STEAM_API_KEY", ""),
        }
    }

    pub fn bind_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("HOST/PORT 不是合法的 SocketAddr")
    }
}

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
