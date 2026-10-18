//! Steam Web API 客户端：在线服务器列表（`IGameServersService/GetServerList`）。
//!
//! 需要 `STEAM_API_KEY`（公共只读 key）。只取 CS（appid 730 / 4465480）且带 KZ 类
//! gametype 标签的服务器；不同社区 tag 不同，逐个标签查询后按 `ip:port` 去重合并。
//! 这是服务器列表的「公开 API」来源；装了本站插件的服务器会用插件上报覆盖它（见 sync）。

use serde::Deserialize;

use super::{SourceClient, SourceError};

const BASE: &str = "https://api.steampowered.com/IGameServersService/GetServerList/v1/";

/// CS 客户端 appid：CS2 为 730，部分老 CSGO 服仍报 4465480。
const APP_IDS: [i64; 2] = [730, 4_465_480];
/// KZ 相关 gametype 标签（`GetServerList` 的 filter 一次只能一个值，故逐个查询）。
const KZ_TAGS: [&str; 3] = ["gokz", "cs2kz", "kz"];
/// 单次查询上限。
const LIMIT: &str = "5000";

#[derive(Debug, Deserialize)]
struct Envelope {
    response: Resp,
}

#[derive(Debug, Default, Deserialize)]
struct Resp {
    #[serde(default)]
    servers: Vec<ServerRow>,
}

/// `ResolveVanityURL` 响应（`success = 1` 才是有效 SteamID64）。
#[derive(Debug, Deserialize)]
struct VanityEnvelope {
    response: VanityResp,
}

#[derive(Debug, Deserialize)]
struct VanityResp {
    #[serde(default)]
    success: i32,
    #[serde(default)]
    steamid: Option<String>,
}

/// `GetServerList` 单条服务器（含实时人数、地图、gametype）。
#[derive(Debug, Clone, Deserialize)]
pub struct ServerRow {
    /// `ip:port`
    pub addr: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub appid: i64,
    #[serde(default)]
    pub players: i32,
    #[serde(default)]
    pub max_players: i32,
    #[serde(default)]
    pub map: String,
    #[serde(default)]
    pub gametype: String,
}

impl ServerRow {
    /// 拆出 `(ip, port)`；端口优先用 addr 里的（与 gameport 一致）。
    pub fn ip_port(&self) -> Option<(String, i32)> {
        let (ip, port) = self.addr.rsplit_once(':')?;
        Some((ip.to_string(), port.parse().ok()?))
    }
}

/// Steam Web API 客户端（薄封装，复用 `SourceClient` 的限速与重试）。
pub struct Steam<'a>(pub &'a SourceClient, pub &'a str);

impl Steam<'_> {
    /// 拉取全部 KZ 类在线服务器（appid × gametype 组合查询 + 去重）。
    pub async fn kz_servers(&self) -> Result<Vec<ServerRow>, SourceError> {
        if self.1.trim().is_empty() {
            return Err(SourceError::Rejected(
                401,
                "未配置 STEAM_API_KEY，跳过 Steam 服务器同步".to_string(),
            ));
        }
        let mut out: Vec<ServerRow> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for appid in APP_IDS {
            for tag in KZ_TAGS {
                let filter = format!("\\appid\\{appid}\\gametype\\{tag}");
                let url = reqwest::Url::parse_with_params(
                    BASE,
                    &[
                        ("key", self.1),
                        ("filter", filter.as_str()),
                        ("limit", LIMIT),
                    ],
                )
                .map_err(|e| SourceError::Decode(e.to_string()))?
                .to_string();
                let env: Envelope = self.0.get_json(&url).await?;
                for s in env.response.servers {
                    if seen.insert(s.addr.clone()) {
                        out.push(s);
                    }
                }
            }
        }
        Ok(out)
    }

    /// 自定义主页名（`steamcommunity.com/id/<vanity>`）→ SteamID64；不存在返回 None。
    pub async fn resolve_vanity(&self, vanity: &str) -> Result<Option<String>, SourceError> {
        if self.1.trim().is_empty() {
            return Err(SourceError::Rejected(
                401,
                "未配置 STEAM_API_KEY，无法解析自定义主页链接".to_string(),
            ));
        }
        let url = reqwest::Url::parse_with_params(
            "https://api.steampowered.com/ISteamUser/ResolveVanityURL/v1/",
            &[("key", self.1), ("vanityurl", vanity)],
        )
        .map_err(|e| SourceError::Decode(e.to_string()))?;
        let env: VanityEnvelope = self.0.get_json(url.as_str()).await?;
        Ok(if env.response.success == 1 {
            env.response.steamid
        } else {
            None
        })
    }
}
