//! 最新记录条目，与前端 `features/records/types.ts` 一一对应。

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct RecordEntry {
    pub player: String,
    /// 用于前端生成头像与跳转资料页。
    pub steam_id64: String,
    /// 真实头像（GOKZ.TOP / Steam CDN）；为空时前端用占位图。
    pub avatar: String,
    /// 用于前端展示地图图片与跳转地图详情。
    pub map_id: i64,
    pub map: String,
    pub mode: String,
    /// 完成耗时（`M:SS.mmm`）。
    pub time: String,
    pub points: i64,
    /// 记录上传时间（UTC ISO8601，前端按本地时区格式化）。
    pub created_at: String,
}
