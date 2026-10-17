//! 外部数据源接入层（PROJECT §9）：**唯一允许发起外部 HTTP 的地方**（见 §3.3）。
//!
//! 约定：
//! - 统一 User-Agent（上游会拒绝默认 UA）、限速闸门与有限重试；
//! - 上游响应类型只在本层出现，不进入各模块 `models.rs`；
//! - 失败向上返回 `SourceError`，由调用方映射为 503（见 §6 错误信封）。

pub mod gokz_top;
pub mod kzgo;

use std::time::Duration;

use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use tokio::time::Instant;

/// 外部数据源错误。
#[derive(Debug)]
pub enum SourceError {
    /// 网络 / 超时 / 5xx：重试后仍失败。
    Unavailable(String),
    /// 4xx：参数错误或被上游拒绝（含 403，多为 UA/鉴权问题）。
    Rejected(u16, String),
    /// 响应无法解析（上游字段变更）。
    Decode(String),
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceError::Unavailable(m) => write!(f, "外部数据源不可用：{m}"),
            SourceError::Rejected(s, m) => write!(f, "外部数据源拒绝（HTTP {s}）：{m}"),
            SourceError::Decode(m) => write!(f, "外部数据源响应解析失败：{m}"),
        }
    }
}

impl std::error::Error for SourceError {}

/// 外部请求客户端：可识别 UA + 单源限速 + 退避重试（PROJECT §9.9）。
pub struct SourceClient {
    http: reqwest::Client,
    /// 上一次请求发出的时刻，用于限速闸门。
    gate: Mutex<Instant>,
    min_interval: Duration,
    max_attempts: u32,
}

const USER_AGENT: &str = "cngokzapi/0.1 (CNGOKZ community site)";

impl SourceClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .build()
            .expect("构建外部请求客户端失败");
        Self {
            http,
            // 初始化为「很久以前」，首次请求不等待
            gate: Mutex::new(Instant::now() - Duration::from_secs(60)),
            min_interval: Duration::from_millis(400),
            max_attempts: 3,
        }
    }

    /// GET 并反序列化为 JSON；5xx/网络错误退避重试，4xx 直接返回。
    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, SourceError> {
        let mut last = String::from("未发起请求");
        for attempt in 0..self.max_attempts {
            if attempt > 0 {
                // 0.5s / 1.5s 退避
                let backoff = 500u64 * 3u64.pow(attempt - 1);
                tokio::time::sleep(Duration::from_millis(backoff)).await;
                tracing::warn!(url = %url, attempt, error = %last, "外部请求失败，重试中");
            }
            self.throttle().await;
            match self.http.get(url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        return resp
                            .json::<T>()
                            .await
                            .map_err(|e| SourceError::Decode(e.to_string()));
                    }
                    let body: String = resp.text().await.unwrap_or_default();
                    let brief: String = body.chars().take(200).collect();
                    if status.is_client_error() {
                        return Err(SourceError::Rejected(status.as_u16(), brief));
                    }
                    last = format!("HTTP {status} {brief}");
                }
                Err(e) => last = e.to_string(),
            }
        }
        Err(SourceError::Unavailable(last))
    }

    /// 限速：保证两次外部请求之间至少间隔 `min_interval`。
    async fn throttle(&self) {
        let mut last = self.gate.lock().await;
        let wait = (*last + self.min_interval).saturating_duration_since(Instant::now());
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
        *last = Instant::now();
    }
}

impl Default for SourceClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        assert!(SourceError::Unavailable("x".into())
            .to_string()
            .contains('x'));
        assert!(SourceError::Rejected(403, "no".into())
            .to_string()
            .contains("403"));
        assert!(SourceError::Decode("bad".into())
            .to_string()
            .contains("bad"));
    }
}
