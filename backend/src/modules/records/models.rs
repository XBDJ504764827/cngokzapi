//! 最新记录条目，与前端 `features/records/types.ts` 一一对应。

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct RecordEntry {
    pub player: String,
    pub map: String,
    pub mode: String,
    pub time: String,
    pub points: u32,
    pub date: String,
}
