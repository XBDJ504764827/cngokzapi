import { http } from '@/services/http'
import type { PlayerProfile } from './types'

export function fetchProfile(steamId64: string): Promise<PlayerProfile> {
  return http.get<PlayerProfile>(`/players/${steamId64}`)
}
