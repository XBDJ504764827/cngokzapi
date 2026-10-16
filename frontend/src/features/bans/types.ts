// 与后端 modules/bans/models.rs 一一对应（snake_case）。

export interface BanEntry {
  player: string
  steam_id64: string
  avatar: string
  reason: string
  banned_at: string
  server: string
  detail: string
}
