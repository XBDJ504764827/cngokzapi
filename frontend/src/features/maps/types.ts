// 与后端 modules/maps/models.rs 一一对应（snake_case）。

export interface MapSummary {
  id: number
  name: string
  tier_kzt: number
  tier_skz: number
  tier_vnl: number
  bonuses: number
  /** 地图制作人显示名（来自上游元数据，可能为空） */
  mappers: string[]
  workshop_id: number | null
  filesize: number
  validated: boolean
}

export interface MapRecordEntry {
  rank: number
  player: string
  steam_id64: string
  avatar: string
  time: string
  points: number
  date: string
}

export interface MapDetail {
  map: MapSummary
  mode: string
  records: MapRecordEntry[]
  /** 本地暂无该图该模式成绩，已入队后台同步（稍后刷新可见） */
  syncing: boolean
}
