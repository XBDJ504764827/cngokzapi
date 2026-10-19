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
  /** 地图 id：用于地图图片与跳转地图详情 */
  map_id: number
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
  /** 国家/地区代码（ISO 3166 alpha-2，如 CN / UA），可能为空 */
  country: string | null
  region: string | null
  /** 本地暂无数据、已触发后台同步 */
  syncing: boolean
  scores: PlayerScoreEntry[]
  recent_records: PlayerRecordEntry[]
}

/** 地图进度条目（已完成 / 未完成）。 */
export interface PlayerMapEntry {
  map_id: number
  name: string
  /** 当前模式的难度（0 = 未评级） */
  tier: number
  /** 已完成主关的最佳耗时；未完成时为 null */
  best_time: string | null
  /** 最佳成绩分数；未完成时为 null */
  points: number | null
  completed: boolean
}

/** 地图进度分页结果。 */
export interface PlayerMapsResult {
  mode: string
  status: 'completed' | 'uncompleted'
  page: number
  per_page: number
  /** 当前状态 + 名称搜索下的总数 */
  total: number
  /** 该模式已完成地图数 */
  completed_total: number
  /** 该模式地图总数 */
  maps_total: number
  data: PlayerMapEntry[]
}
