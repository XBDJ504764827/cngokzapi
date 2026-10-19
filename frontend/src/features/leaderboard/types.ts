// 与后端 modules/leaderboard/models.rs 一一对应（snake_case）。
// 说明：rank 为**当前排序下的名次**（跨越分页连续，1 起）。

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

export interface LeaderboardPagination {
  page: number
  per_page: number
  total: number
}

export interface LeaderboardPage {
  data: LeaderboardEntry[]
  pagination: LeaderboardPagination
}

export type ActivityFilter = 'all' | '7' | '30' | 'inactive'

/** 后端排序字段：points / maps / avg / rating。 */
export type SortKey = 'points' | 'maps' | 'avg' | 'rating'

export interface LeaderboardParams {
  page: number
  per_page: number
  query?: string
  sort?: SortKey
  activity?: ActivityFilter
}
