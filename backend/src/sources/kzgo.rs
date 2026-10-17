//! kzgo.eu 公开 API 客户端（兜底数据源，见 PROJECT §9.2）。
//! 目前只用于补 gokz.top 缺失的**地图制作人显示名**（`mapperNames`）。

use serde::Deserialize;

use super::{SourceClient, SourceError};

pub const BASE: &str = "https://kzgo.eu/api";

#[derive(Debug, Clone, Deserialize)]
pub struct MapMeta {
    pub name: String,
    /// 上游可能给 null（实测 kz_south 为 ["nopey", null]），故用 Option 兜住。
    #[serde(rename = "mapperNames", default)]
    pub mapper_names: Vec<Option<String>>,
    #[serde(rename = "mapperIds", default)]
    pub mapper_ids: Vec<Option<String>>,
    #[serde(default)]
    pub bonuses: i16,
    #[serde(default)]
    pub sp: bool,
    #[serde(default)]
    pub vp: bool,
}

/// kzgo 客户端（复用 `SourceClient` 的限速与重试）。
pub struct Kzgo<'a>(pub &'a SourceClient);

impl Kzgo<'_> {
    /// 全量地图元数据（约 1000 条，一次拉完）。
    pub async fn maps(&self) -> Result<Vec<MapMeta>, SourceError> {
        self.0.get_json(&format!("{BASE}/maps")).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_camel_case_fields() {
        // 第二个条目复现上游的 null 制作人（曾导致整批解析失败）
        let raw = r#"[{"name":"bkz_apricity_v3","id":200,"tier":5,"bonuses":1,
            "mapperNames":["Badges"],"mapperIds":["76561197983438223"],"sp":false,"vp":false},
            {"name":"kz_south","id":1,"tier":3,"bonuses":0,
            "mapperNames":["nopey",null],"mapperIds":[null],"sp":false,"vp":false}]"#;
        let rows: Vec<MapMeta> = serde_json::from_str(raw).unwrap();
        assert_eq!(rows[0].name, "bkz_apricity_v3");
        assert_eq!(
            rows[0]
                .mapper_names
                .clone()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            vec!["Badges".to_string()]
        );
        assert_eq!(rows[0].bonuses, 1);
        assert_eq!(rows[1].mapper_names.iter().flatten().count(), 1);
    }
}
