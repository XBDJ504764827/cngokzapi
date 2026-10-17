// 与后端 modules/stats/models.rs 一一对应（snake_case）。

export interface HomeStats {
  players: number
  maps: number
  records: number
  servers_online: number
  players_online: number
  bans: number
  updated_at: string
}
