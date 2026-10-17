//! 上报接收端的响应体（PROJECT §9.6）。

use serde::Serialize;

/// 原生接口响应：逐条返回结果，避免"整批失败"导致客户端重试全部数据。
#[derive(Serialize)]
pub struct IngestResponse {
    pub results: Vec<IngestResult>,
}

#[derive(Serialize)]
pub struct IngestResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_uuid: Option<String>,
    /// `accepted`（已入库）/ `duplicate`（已存在，幂等命中）/ `rejected`（被拒）。
    pub status: &'static str,
    /// 该成绩在其榜单（同地图/模式/关卡/传送口径）中的名次。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place: Option<i64>,
    /// 单条成绩分：本站 GOKZ 公式尚未落地（PROJECT §9.4 第 1 层），暂为 null。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl IngestResult {
    pub fn accepted(record_uuid: Option<String>, place: i64) -> Self {
        Self {
            record_uuid,
            status: "accepted",
            place: Some(place),
            points: None,
            reason: None,
        }
    }

    pub fn duplicate(record_uuid: Option<String>) -> Self {
        Self {
            record_uuid,
            status: "duplicate",
            place: None,
            points: None,
            reason: None,
        }
    }

    pub fn rejected(record_uuid: Option<String>, reason: impl Into<String>) -> Self {
        Self {
            record_uuid,
            status: "rejected",
            place: None,
            points: None,
            reason: Some(reason.into()),
        }
    }
}

/// GlobalAPI 兼容层响应：官方客户端用这三个字段播报"新纪录"。
#[derive(Serialize)]
pub struct CompatResponse {
    /// 同榜单（含传送口径）名次。
    pub place: i64,
    /// 与官方口径对齐：同榜单名次。
    pub top_place: i64,
    /// 不分传送口径的名次。
    pub top_overall_place: i64,
}
