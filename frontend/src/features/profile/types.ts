// 与后端 modules/players/models.rs 一一对应（snake_case）。

export interface PlayerScoreEntry {
  mode: string
  /** 上游全球排名（按 rating），个人资料页展示；榜单页用本站名次 */
  rank: number | null
  points: number
  points_local: number | null
  rating: number | null
  maps_completed: number
  wrs_nub: number
  wrs_pro: number
  records: number
}

export interface PlayerRecordEntry {
  map: string
  mode: string
  stage: number
  time: string
  points: number
  date: string
}

export interface PlayerProfile {
  steam_id64: string
  name: string
  avatar: string
  region: string | null
  /** 本地暂无数据、已触发后台同步 */
  syncing: boolean
  scores: PlayerScoreEntry[]
  recent_records: PlayerRecordEntry[]
}
