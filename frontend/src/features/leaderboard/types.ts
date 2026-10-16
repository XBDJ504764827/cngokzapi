// 与后端 modules/leaderboard/models.rs 一一对应（snake_case）。

export interface LeaderboardEntry {
  rank: number
  player: string
  steam_id64: string
  avatar: string
  maps_completed: number
  total_points: number
  avg_points: number
  records: number
  last_played: string
  last_played_days_ago: number
}

export type ActivityFilter = 'all' | '7' | '30' | 'inactive'

export type SortKey = 'total_points' | 'maps_completed' | 'avg_points' | 'records'
