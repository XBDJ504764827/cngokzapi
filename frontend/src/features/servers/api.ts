import { http } from '@/services/http'
import type { ServerEntry } from './types'

export function fetchServers(): Promise<ServerEntry[]> {
  return http.get<ServerEntry[]>('/servers')
}
