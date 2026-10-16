import { http } from '@/services/http'
import type { Mode } from '@/types/common'
import type { LeaderboardEntry } from './types'

export function fetchLeaderboard(mode: Mode): Promise<LeaderboardEntry[]> {
  return http.get<LeaderboardEntry[]>(`/leaderboard?mode=${mode}`)
}
