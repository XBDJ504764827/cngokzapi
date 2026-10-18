//! KZTimer Global API 客户端（PROJECT §9.2：成绩/封禁的官方数据源）。
//!
//! 官方接口是公开广播式的：`/records/top/recent` 增量拉取即可拿到最新成绩。
//! 模式：`kz_timer`=KZT、`kz_simple`=SKZ、`kz_vanilla`=VNL。
//! 注意：`/records/top`、`/records/top/recent` 不支持按模式过滤（实测忽略参数），
//! 需拉回后在 `modules/sync` 里按 `mode` 过滤；时间字段 `created_on` 为**无时区 UTC**。

use serde::Deserialize;

use super::{SourceClient, SourceError};

pub const BASE: &str = "https://kztimerglobal.com/api/v2.0";

/// 拼接带查询参数的 URL（参数经 URL 编码）。
pub fn url(path: &str, params: &[(&str, String)]) -> String {
    let mut u = reqwest::Url::parse(&format!("{BASE}{path}")).expect("KZTimer 地址常量非法");
    if !params.is_empty() {
        u.query_pairs_mut()
            .extend_pairs(params.iter().map(|(k, v)| (*k, v.as_str())));
    }
    u.to_string()
}

/// 上游模式名 → 本站模式（PROJECT 的 KZT/SKZ/VNL）。
pub fn mode_name(mode: &str) -> Option<&'static str> {
    match mode {
        "kz_timer" => Some("KZT"),
        "kz_simple" => Some("SKZ"),
        "kz_vanilla" => Some("VNL"),
        _ => None,
    }
}

/// `/records/top`、`/records/top/recent` 行（成绩字段一致）。
#[derive(Debug, Clone, Deserialize)]
pub struct RecordRow {
    pub id: i64,
    pub steamid64: String,
    #[serde(default)]
    pub player_name: String,
    #[serde(default)]
    pub server_id: Option<i64>,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub map_id: Option<i64>,
    #[serde(default)]
    pub map_name: Option<String>,
    #[serde(default)]
    pub stage: i32,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub tickrate: i32,
    pub time: f64,
    #[serde(default)]
    pub teleports: i32,
    #[serde(default)]
    pub points: i32,
    #[serde(default)]
    pub points_overall: i32,
    pub created_on: String,
}

/// `/maps` 行（注意：只有一个 `difficulty`，没有三模式 tier / bonus 数）。
#[derive(Debug, Clone, Deserialize)]
pub struct MapRow {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub filesize: i64,
    #[serde(default)]
    pub validated: bool,
    #[serde(default)]
    pub difficulty: i16,
    #[serde(default)]
    pub workshop_url: Option<String>,
    #[serde(default)]
    pub created_on: Option<String>,
}

/// `/bans` 行。
#[derive(Debug, Clone, Deserialize)]
pub struct BanRow {
    pub id: i64,
    #[serde(default)]
    pub ban_type: Option<String>,
    #[serde(default)]
    pub expires_on: Option<String>,
    pub steamid64: String,
    #[serde(default)]
    pub player_name: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub stats: Option<String>,
    #[serde(default)]
    pub server_id: Option<i64>,
    pub created_on: String,
}

/// KZTimer 客户端。
pub struct Kztimer<'a>(pub &'a SourceClient);

impl Kztimer<'_> {
    /// 最新成绩（广播式增量源；不支持按模式过滤，调用方自行过滤）。
    pub async fn recent_records(&self, limit: i64) -> Result<Vec<RecordRow>, SourceError> {
        self.0
            .get_json(&url("/records/top/recent", &[("limit", limit.to_string())]))
            .await
    }

    /// 某地图某关卡的成绩（按时间升序；调用方按模式过滤）。
    pub async fn map_records(
        &self,
        map_name: &str,
        stage: i32,
        limit: i64,
    ) -> Result<Vec<RecordRow>, SourceError> {
        self.0
            .get_json(&url(
                "/records/top",
                &[
                    ("map_name", map_name.to_string()),
                    ("stage", stage.to_string()),
                    ("limit", limit.to_string()),
                ],
            ))
            .await
    }

    /// 某玩家的成绩（个人资料懒加载用）。
    pub async fn player_records(
        &self,
        steamid64: &str,
        limit: i64,
    ) -> Result<Vec<RecordRow>, SourceError> {
        self.0
            .get_json(&url(
                "/records/top",
                &[
                    ("steamid64", steamid64.to_string()),
                    ("limit", limit.to_string()),
                ],
            ))
            .await
    }

    /// 封禁名单（倒序）。
    pub async fn bans(&self, limit: i64, offset: i64) -> Result<Vec<BanRow>, SourceError> {
        self.0
            .get_json(&url(
                "/bans",
                &[("limit", limit.to_string()), ("offset", offset.to_string())],
            ))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_builder_encodes_params() {
        let u = url("/records/top", &[("map_name", "bkz_a b".to_string())]);
        assert!(u.starts_with(BASE));
        assert!(u.contains("map_name=bkz_a+b") || u.contains("map_name=bkz_a%20b"));
    }

    #[test]
    fn mode_mapping() {
        assert_eq!(mode_name("kz_timer"), Some("KZT"));
        assert_eq!(mode_name("kz_simple"), Some("SKZ"));
        assert_eq!(mode_name("kz_vanilla"), Some("VNL"));
        assert_eq!(mode_name("unknown"), None);
    }
}
