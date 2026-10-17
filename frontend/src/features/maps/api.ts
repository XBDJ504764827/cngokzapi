import { http } from '@/services/http'
import type { MapDetail, MapSummary } from './types'

export interface MapListParams {
  query?: string
  tier?: number
  /** 按哪个模式的 tier 过滤；缺省按综合 tier */
  mode?: string
}

export function fetchMaps(params: MapListParams = {}): Promise<MapSummary[]> {
  const q = new URLSearchParams()
  if (params.query) q.set('query', params.query)
  if (params.tier) q.set('tier', String(params.tier))
  if (params.mode) q.set('mode', params.mode)
  const s = q.toString()
  return http.get<MapSummary[]>(`/maps${s ? `?${s}` : ''}`)
}

export function fetchMapDetail(id: number, mode: string): Promise<MapDetail> {
  return http.get<MapDetail>(`/maps/${id}?mode=${mode}`)
}
