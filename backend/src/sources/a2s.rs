//! A2S 直连查询（Source Server Query，UDP）。
//!
//! Steam Web API `GetServerList` 只返回人数、不返回昵称；这里直接向服务器发
//! `A2S_INFO` / `A2S_PLAYER`，用于补齐在线玩家昵称与更实时的人数。
//! 单台服失败（超时/禁用查询）返回 `None`，由调用方回退到 gokz.top。

use std::time::Duration;

use a2s::A2SClient;

/// 一次 A2S 查询结果。
pub struct A2sSnapshot {
    pub players: i32,
    pub max_players: i32,
    /// 玩家昵称列表（已过滤 GOTV/SourceTV 与控制字符）。
    pub names: Vec<String>,
    /// `A2S_PLAYER` 是否成功；失败时昵称可能为空，可让上层回退。
    pub players_ok: bool,
}

/// 查询单台服务器；任何失败都返回 `None`（不阻塞整轮同步）。
pub async fn query(ip: &str, port: i32, timeout: Duration) -> Option<A2sSnapshot> {
    let addr = format!("{ip}:{port}");
    let mut client = A2SClient::new().await.ok()?;
    client.set_timeout(timeout).ok()?;

    let info = client.info(addr.as_str()).await.ok()?;
    let (names, players_ok) = match client.players(addr.as_str()).await {
        Ok(players) => {
            let names = players
                .into_iter()
                .map(|p| p.name)
                .map(|n| sanitize(&n))
                .filter(|n| !n.is_empty() && !is_broadcast(n))
                .collect();
            (names, true)
        }
        Err(_) => (Vec::new(), false),
    };

    Some(A2sSnapshot {
        players: i32::from(info.players),
        max_players: i32::from(info.max_players),
        names,
        players_ok,
    })
}

/// 去掉颜色码/控制字符与零宽字符，保留正常昵称。
fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '\u{200b}'..='\u{200f}' | '\u{feff}'))
        .collect();
    cleaned.trim().to_string()
}

/// 观战/电视端不算真实玩家。
fn is_broadcast(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "gotv" || lower == "sourcetv" || lower.contains("sourcetv")
}

#[cfg(test)]
mod tests {
    use super::{is_broadcast, sanitize};

    #[test]
    fn sanitizes_color_codes_and_zero_width() {
        assert_eq!(sanitize("\u{1}Player\u{200b}Name\u{1f}"), "PlayerName");
        assert_eq!(sanitize("  normal  "), "normal");
    }

    #[test]
    fn filters_broadcast_clients() {
        assert!(is_broadcast("GOTV"));
        assert!(is_broadcast("SourceTV"));
        assert!(!is_broadcast("Player"));
    }
}
