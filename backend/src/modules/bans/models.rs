//! 封禁条目，与前端 `features/bans/types.ts` 一一对应。

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct BanEntry {
    pub player: String,
    pub steam_id64: String,
    pub avatar: String,
    pub reason: String,
    pub banned_at: String,
    pub server: String,
    pub detail: String,
}
