// 与后端 modules/records/models.rs 一一对应（snake_case）。

export interface RecordEntry {
  player: string
  steam_id64: string
  /** 地图 id，用于展示地图图片与跳转地图详情 */
  map_id: number
  map: string
  mode: string
  /** 完成耗时（M:SS.mmm） */
  time: string
  points: number
  /** 记录上传时间（UTC ISO8601） */
  created_at: string
}
