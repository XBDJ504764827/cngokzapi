// 与后端 modules/leaderboard/models.rs 一一对应（snake_case）。
// 说明：rank 为**本站**名次（按本站汇总分排序，见 PROJECT §9.4）。

export interface LeaderboardEntry {
  rank: number | null
  player: string
  steam_id64: string
  avatar: string
  maps_completed: number
  total_points: number
  avg_points: number
  records: number
  /** 本地已同步成绩的最近游玩日期；null = 尚未同步到该玩家成绩 */
  last_played: string | null
  last_played_days_ago: number | null
}

export type ActivityFilter = 'all' | '7' | '30' | 'inactive'

export type SortKey = 'total_points' | 'maps_completed' | 'avg_points' | 'records'
