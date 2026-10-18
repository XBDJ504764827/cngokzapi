// 与后端 modules/servers/models.rs 一一对应（snake_case）。
// 注意：ip/port 仅用于「复制 IP」按钮，页面不渲染明文地址（PROJECT §2 防爬虫约束）。

export interface ServerEntry {
  id: number
  name: string
  map: string
  ip: string
  port: number
  player_count: number
  max_players: number
  country: string | null
  city: string | null
  region: string | null
  group_name: string | null
  /** 社区分类（后端按 group_name / 服务器名归一化，如 CNGOKZ、AXE、HIME）。 */
  category: string
  online: boolean
  /** 当前地图难度（0 = 未知） */
  map_tier: number
  /** 实时在线玩家昵称 */
  player_names: string[]
}
