//! 最新记录条目，与前端 `features/records/types.ts` 一一对应。

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct RecordEntry {
    pub player: String,
    /// 用于前端生成头像与跳转资料页。
    pub steam_id64: String,
    pub map: String,
    pub mode: String,
    pub time: String,
    pub points: i64,
    pub date: String,
}
