import { http } from '@/services/http'
import type { PlayerMapsResult, PlayerProfile } from './types'

export function fetchProfile(steamId64: string): Promise<PlayerProfile> {
  return http.get<PlayerProfile>(`/players/${steamId64}`)
}

export interface PlayerMapsParams {
  mode: string
  status: 'completed' | 'uncompleted'
  page: number
  per_page: number
  q?: string
}

/** 玩家「地图进度」：已完成 / 未完成地图（分页 + 名称搜索）。 */
export function fetchPlayerMaps(
  steamId64: string,
  params: PlayerMapsParams,
): Promise<PlayerMapsResult> {
  const qs = new URLSearchParams({
    mode: params.mode,
    status: params.status,
    page: String(params.page),
    per_page: String(params.per_page),
  })
  if (params.q) qs.set('q', params.q)
  return http.get<PlayerMapsResult>(`/players/${steamId64}/maps?${qs.toString()}`)
}
