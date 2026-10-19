import { http } from '@/services/http'
import type { Mode } from '@/types/common'
import type { LeaderboardPage, LeaderboardParams } from './types'

/** 分页榜单：搜索 / 活跃度 / 排序由服务端处理。 */
export function fetchLeaderboard(
  mode: Mode,
  params: LeaderboardParams,
): Promise<LeaderboardPage> {
  const qs = new URLSearchParams({
    mode,
    page: String(params.page),
    per_page: String(params.per_page),
  })
  if (params.query) qs.set('query', params.query)
  if (params.sort) qs.set('sort', params.sort)
  if (params.activity) qs.set('activity', params.activity)
  return http.get<LeaderboardPage>(`/leaderboard?${qs.toString()}`)
}
