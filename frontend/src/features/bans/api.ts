import { http } from '@/services/http'
import type { PageResult } from '@/types/common'
import type { BanEntry } from './types'

export function fetchBans(query: string, page: number, perPage = 20): Promise<PageResult<BanEntry>> {
  const q = query.trim()
  return http.get<PageResult<BanEntry>>(
    `/bans?query=${encodeURIComponent(q)}&page=${page}&per_page=${perPage}`,
  )
}
