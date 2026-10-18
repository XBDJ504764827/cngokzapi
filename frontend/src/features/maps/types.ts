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
  /** 当前模式下主关最快成绩（WR）保持者；null = 本地暂无该模式成绩 */
  wr_player: string | null
  /** WR 耗时（M:SS.mmm） */
  wr_time: string | null
  /** WR 记录日期 YYYY-MM-DD */
  wr_date: string | null
}

export interface MapRecordEntry {
  rank: number
  player: string
  steam_id64: string
  avatar: string
  /** 完成耗时 M:SS.mmm */
  time: string
  points: number
  /** 存读点次数（PRO 恒为 0） */
  teleports: number
  /** 记录上传时间（UTC ISO8601） */
  created_at: string
}

/** 单模式的两张榜：PRO = 无存读点，TP = 有存读点。 */
export interface ModeBoard {
  mode: string
  wr_pro: MapRecordEntry | null
  wr_tp: MapRecordEntry | null
  pro: MapRecordEntry[]
  tp: MapRecordEntry[]
}

export interface MapDetail {
  map: MapSummary
  boards: ModeBoard[]
  /** 任一模式整图同步尚未完成 */
  syncing: boolean
}
