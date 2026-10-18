import { http } from '@/services/http'

export interface ResolvedPlayer {
  steam_id64: string
}

/** 统一解析 SteamID / SteamID64 / SteamID3 / Steam 个人主页链接。 */
export function resolvePlayer(q: string): Promise<ResolvedPlayer> {
  return http.get<ResolvedPlayer>(`/players/resolve?q=${encodeURIComponent(q)}`)
}
