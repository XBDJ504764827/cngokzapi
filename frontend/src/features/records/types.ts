// 与后端 modules/records/models.rs 一一对应（snake_case）。

export interface RecordEntry {
  player: string
  steam_id64: string
  map: string
  mode: string
  time: string
  points: number
  date: string
}
