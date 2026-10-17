//! GOKZ.TOP 公开 API 客户端（主数据源，见 PROJECT §9.2）。
//! 仅做「取数 + 反序列化」，字段映射与落库在 `modules/sync`。

use serde::Deserialize;

use super::{SourceClient, SourceError};

pub const BASE: &str = "https://api.gokz.top/v1";

/// 拼接带查询参数的 URL（参数经 URL 编码）。
fn url(path: &str, params: &[(&str, String)]) -> String {
    let mut u = reqwest::Url::parse(&format!("{BASE}{path}")).expect("GOKZ.TOP 地址常量非法");
    if !params.is_empty() {
        u.query_pairs_mut()
            .extend_pairs(params.iter().map(|(k, v)| (*k, v.as_str())));
    }
    u.to_string()
}

/// 通用分页信封 `{ data, count }`。
#[derive(Debug, Clone, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub count: Option<i64>,
}

/// 地图三模式 tier（`VNL` 为 0 表示该模式未评级）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Tiers {
    #[serde(rename = "OVR", default)]
    pub ovr: i16,
    #[serde(rename = "KZT", default)]
    pub kzt: i16,
    #[serde(rename = "SKZ", default)]
    pub skz: i16,
    #[serde(rename = "VNL", default)]
    pub vnl: i16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerRef {
    pub steamid64: String,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroupRef {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub custom_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapRef {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub tier: Option<i16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerRef {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub group: Option<GroupRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapRow {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub filesize: i64,
    #[serde(default)]
    pub validated: bool,
    #[serde(default)]
    pub tiers: Tiers,
    #[serde(default)]
    pub bonus_count: i16,
    #[serde(default)]
    pub workshop_id: Option<i64>,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
}

/// `GET /leaderboards/players` 行。
#[derive(Debug, Clone, Deserialize)]
pub struct LeaderRow {
    #[serde(default)]
    pub rank: Option<i32>,
    #[serde(default)]
    pub global_rank: Option<i32>,
    pub player: PlayerRef,
    #[serde(default)]
    pub points: i64,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub unique_map_finishes: i32,
    #[serde(default)]
    pub wrs_nub: i32,
    #[serde(default)]
    pub wrs_pro: i32,
    #[serde(default)]
    pub records_900_plus: i32,
    #[serde(default)]
    pub records_800_plus: i32,
}

/// `GET /leaderboards/players/{steamid64}`（单人档案，多 `region` / 区域排名）。
#[derive(Debug, Clone, Deserialize)]
pub struct PlayerRow {
    #[serde(default)]
    pub rank: Option<i32>,
    #[serde(default)]
    pub global_rank: Option<i32>,
    #[serde(default)]
    pub region: Option<String>,
    pub player: PlayerRef,
    #[serde(default)]
    pub points: i64,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub unique_map_finishes: i32,
    #[serde(default)]
    pub wrs_nub: i32,
    #[serde(default)]
    pub wrs_pro: i32,
    #[serde(default)]
    pub records_900_plus: i32,
    #[serde(default)]
    pub records_800_plus: i32,
}

/// `GET /records`（可按 map_name / steamid64 过滤）行。
#[derive(Debug, Clone, Deserialize)]
pub struct TopRecordRow {
    pub uuid: String,
    pub player: PlayerRef,
    #[serde(default)]
    pub map_id: Option<i64>,
    #[serde(default)]
    pub map_name: Option<String>,
    #[serde(default)]
    pub map_tier: Option<i16>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub mode_id: Option<i32>,
    #[serde(default)]
    pub stage: i32,
    #[serde(default)]
    pub tickrate: i32,
    #[serde(default)]
    pub teleports: i32,
    pub time: f64,
    #[serde(default)]
    pub points: i32,
    #[serde(default)]
    pub server_id: Option<i64>,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub wr_time: Option<f64>,
    pub created_on: String,
    #[serde(default)]
    pub is_valid: Option<bool>,
}

/// `GET /records/recent` 行（地图/模式/服务器为嵌套对象）。
#[derive(Debug, Clone, Deserialize)]
pub struct RecentRecordRow {
    pub uuid: String,
    pub player: PlayerRef,
    #[serde(default)]
    pub map: Option<MapRef>,
    #[serde(default)]
    pub mode: Option<ModeRef>,
    #[serde(default)]
    pub server: Option<ServerRef>,
    #[serde(default)]
    pub stage: i32,
    #[serde(default)]
    pub teleports: i32,
    pub time: f64,
    #[serde(default)]
    pub points: i32,
    pub created_on: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModeRef {
    pub id: i32,
    pub name: String,
}

/// 服务器上的在线玩家（`live_status.players[]`，只取昵称）。
/// 上游可能给 null（字段或昵称都可能），故全部用 Option 兜住。
#[derive(Debug, Clone, Deserialize)]
pub struct LivePlayer {
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiveStatus {
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub map: Option<String>,
    #[serde(default)]
    pub player_count: Option<i32>,
    #[serde(default)]
    pub max_players: Option<i32>,
    #[serde(default)]
    pub is_online: Option<bool>,
    /// 在线玩家列表（服务器卡片显示玩家标签用）。
    #[serde(default)]
    pub players: Option<Vec<LivePlayer>>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerRow {
    pub id: String,
    pub ip: String,
    pub port: i32,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub is_public: Option<bool>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub group: Option<GroupRef>,
    #[serde(default)]
    pub live_status: Option<LiveStatus>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanRow {
    pub uuid: String,
    #[serde(default)]
    pub ban_type: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub stats: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub server: Option<ServerRef>,
    pub player: PlayerRef,
    pub created_at: String,
}

/// GOKZ.TOP 客户端（薄封装，复用 `SourceClient` 的限速与重试）。
pub struct GokzTop<'a>(pub &'a SourceClient);

impl GokzTop<'_> {
    /// 全量地图（当前约 1100 张，一次拉完）。
    pub async fn maps(&self) -> Result<Vec<MapRow>, SourceError> {
        self.0
            .get_json(&url("/maps", &[("limit", "5000".to_string())]))
            .await
    }

    /// 某模式的玩家榜（`scope` 为 KZT/SKZ/VNL）。
    pub async fn leaderboard(
        &self,
        mode: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Page<LeaderRow>, SourceError> {
        self.0
            .get_json(&url(
                "/leaderboards/players",
                &[
                    ("scope", mode.to_string()),
                    ("limit", limit.to_string()),
                    ("offset", offset.to_string()),
                    ("include_count", "true".to_string()),
                ],
            ))
            .await
    }

    /// 单个玩家在某模式的排名档案；未上榜返回 `None`。
    pub async fn player(
        &self,
        steamid64: &str,
        mode: &str,
    ) -> Result<Option<PlayerRow>, SourceError> {
        let u = url(
            &format!("/leaderboards/players/{steamid64}"),
            &[("scope", mode.to_string())],
        );
        match self.0.get_json::<PlayerRow>(&u).await {
            Ok(row) => Ok(Some(row)),
            Err(SourceError::Rejected(404, _)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 某地图某关卡的成绩（`stage=0` 为主关，>0 为 bonus），按时间升序。
    pub async fn map_records(
        &self,
        map_name: &str,
        stage: i32,
        limit: i64,
    ) -> Result<Page<TopRecordRow>, SourceError> {
        self.0
            .get_json(&url(
                "/records",
                &[
                    ("map_name", map_name.to_string()),
                    ("stage", stage.to_string()),
                    ("limit", limit.to_string()),
                ],
            ))
            .await
    }

    /// 某玩家的成绩（用于个人资料懒加载）。
    pub async fn player_records(
        &self,
        steamid64: &str,
        limit: i64,
    ) -> Result<Page<TopRecordRow>, SourceError> {
        self.0
            .get_json(&url(
                "/records",
                &[
                    ("steamid64", steamid64.to_string()),
                    ("limit", limit.to_string()),
                ],
            ))
            .await
    }

    /// 最新记录（按模式）。
    pub async fn recent_records(
        &self,
        mode: &str,
        limit: i64,
    ) -> Result<Page<RecentRecordRow>, SourceError> {
        self.0
            .get_json(&url(
                "/records/recent",
                &[("mode", mode.to_string()), ("limit", limit.to_string())],
            ))
            .await
    }

    /// 在线公开服务器（含 live_status：当前地图与人数）。
    pub async fn servers(&self) -> Result<Page<ServerRow>, SourceError> {
        self.0
            .get_json(&url(
                "/servers",
                &[("limit", "500".to_string()), ("online", "true".to_string())],
            ))
            .await
    }

    /// 封禁名单（倒序）。
    pub async fn bans(&self, limit: i64, offset: i64) -> Result<Page<BanRow>, SourceError> {
        self.0
            .get_json(&url(
                "/bans",
                &[
                    ("limit", limit.to_string()),
                    ("offset", offset.to_string()),
                    ("include_count", "true".to_string()),
                ],
            ))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_builder_encodes_params() {
        let u = url("/records", &[("map_name", "bkz_a b".to_string())]);
        assert!(u.starts_with("https://api.gokz.top/v1/records?"));
        assert!(u.contains("map_name=bkz_a+b") || u.contains("map_name=bkz_a%20b"));
    }

    #[test]
    fn decodes_leader_row() {
        let raw = r#"{"rank":1,"global_rank":1,"player":{"steamid64":"76561198325578948","display_name":"x"},
            "points":1670803,"rating":10.9,"unique_map_finishes":967,"wrs_nub":666,"wrs_pro":47,
            "records_900_plus":733,"records_800_plus":157}"#;
        let row: LeaderRow = serde_json::from_str(raw).unwrap();
        assert_eq!(row.points, 1_670_803);
        assert_eq!(row.unique_map_finishes, 967);
        assert_eq!(row.player.steamid64, "76561198325578948");
    }

    #[test]
    fn decodes_bare_map_list_and_tiers() {
        let raw = r#"[{"id":200,"name":"bkz_apricity_v3","filesize":58,"validated":true,
            "tiers":{"OVR":5,"KZT":5,"SKZ":7,"VNL":0},"bonus_count":1,"workshop_id":1986459033,
            "created_on":"2018-01-09T10:45:50Z","authors":["1"]}]"#;
        let rows: Vec<MapRow> = serde_json::from_str(raw).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].tiers.skz, 7);
        assert_eq!(rows[0].tiers.vnl, 0);
        assert_eq!(rows[0].bonus_count, 1);
    }

    #[test]
    fn decodes_server_live_players_with_nulls() {
        // 复现真实上游：players 有昵称，也可能 name 为 null；另有服务器 players 为 null
        let raw = r#"{"data":[
          {"id":"s1","ip":"1.2.3.4","port":27015,"status":"enabled","is_public":true,
           "live_status":{"hostname":"AXE GOKZ #B01","map":"kz_sp1_saishuu","player_count":2,
             "max_players":20,"is_online":true,
             "players":[{"name":"Chilldog"},{"name":null}],
             "updated_at":"2026-10-17T12:34:05Z"}},
          {"id":"s2","ip":"1.2.3.5","port":27015,"is_public":null,
           "live_status":{"hostname":null,"map":null,"player_count":null,"max_players":null,
             "is_online":null,"players":null,"updated_at":null}}
        ],"count":2}"#;
        let page: Page<ServerRow> = serde_json::from_str(raw).unwrap();
        let first = &page.data[0];
        let live = first.live_status.as_ref().unwrap();
        assert_eq!(live.player_count, Some(2));
        let names: Vec<String> = live
            .players
            .as_ref()
            .unwrap()
            .iter()
            .filter_map(|p| p.name.clone())
            .collect();
        assert_eq!(names, vec!["Chilldog".to_string()]);
        assert_eq!(first.is_public, Some(true));

        // 全 null 的服务器不应导致解析失败
        let second = &page.data[1];
        assert_eq!(second.is_public, None);
        assert!(second.live_status.as_ref().unwrap().players.is_none());
    }

    #[test]
    fn decodes_recent_record_nested_refs() {
        let raw = r#"{"data":[{"uuid":"u","player":{"steamid64":"1","display_name":"p"},
            "map":{"id":805,"name":"kz_antimony","tier":4},
            "server":{"id":1394,"name":"ANDEzz","group":{"name":"CNGOKZ","custom_id":"cngokz"}},
            "mode":{"id":200,"name":"KZT"},"stage":0,"teleports":91,"time":431.531,"points":0,
            "created_on":"2026-10-17T12:33:57Z"}],"count":1}"#;
        let page: Page<RecentRecordRow> = serde_json::from_str(raw).unwrap();
        let r = &page.data[0];
        assert_eq!(r.map.as_ref().unwrap().name, "kz_antimony");
        assert_eq!(r.mode.as_ref().unwrap().name, "KZT");
        assert_eq!(
            r.server
                .as_ref()
                .unwrap()
                .group
                .as_ref()
                .unwrap()
                .custom_id
                .as_deref(),
            Some("cngokz")
        );
    }
}
