import { http } from '@/services/http'
import type { HomeStats } from './types'

export function fetchStats(): Promise<HomeStats> {
  return http.get<HomeStats>('/stats')
}
