import { http } from '@/services/http'
import type { RecordEntry } from './types'

export function fetchRecords(): Promise<RecordEntry[]> {
  return http.get<RecordEntry[]>('/records')
}
