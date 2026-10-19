// 输入格式识别：仅用于查询页的即时提示与 SteamID64 预览，
// 真正解析以后端 `/api/v1/players/resolve` 为准（规则与其保持一致）。

export type QueryKind =
  | 'steamid64'
  | 'steam2'
  | 'steam3'
  | 'account'
  | 'profile'
  | 'vanity'
  | 'unknown'

export interface QueryDetection {
  kind: QueryKind
  /** 展示用的格式名称；无法识别时为 null。 */
  label: string | null
  /** 可直接推算出的 SteamID64；自定义主页链接需后端调用 Steam，返回 null。 */
  steamId64: string | null
}

const STEAM64_BASE = 76561197960265728n

function isSteam64(s: string): boolean {
  return /^\d{17}$/.test(s)
}

/** `STEAM_x:y:z` → SteamID64（`base + z * 2 + y`）。 */
function parseSteam2(s: string): string | null {
  const m = /^steam_\d+:([01]):(\d+)$/i.exec(s)
  if (!m) return null
  return (STEAM64_BASE + BigInt(m[2]) * 2n + BigInt(m[1])).toString()
}

/** `[U:1:w]` / `U:1:w` → SteamID64（`base + w`）。 */
function parseSteam3(s: string): string | null {
  const m = /^\[?u:1:(\d+)\]?$/i.exec(s)
  if (!m) return null
  return (STEAM64_BASE + BigInt(m[1])).toString()
}

/** 纯数字账号 id（1-10 位，SteamID3 数字形式）→ SteamID64。 */
function parseAccountId(s: string): string | null {
  if (!/^\d{1,10}$/.test(s)) return null
  return (STEAM64_BASE + BigInt(s)).toString()
}

interface ProfileRef {
  vanity: boolean
  value: string
}

/** Steam 个人主页链接：`/profiles/<SteamID64>` 或 `/id/<自定义名>`。 */
function parseProfileUrl(s: string): ProfileRef | null {
  let url: URL
  try {
    url = new URL(s.startsWith('http://') || s.startsWith('https://') ? s : `https://${s}`)
  } catch {
    return null
  }
  const host = url.host.toLowerCase()
  if (host !== 'steamcommunity.com' && !host.endsWith('.steamcommunity.com')) return null
  const segments = url.pathname.split('/').filter((p) => p.length > 0)
  if (segments.length !== 2) return null
  const [head, tail] = segments
  if (head === 'profiles' && isSteam64(tail)) return { vanity: false, value: tail }
  if (head === 'id' && tail.length <= 64) return { vanity: true, value: tail }
  return null
}

/** 识别输入格式；空输入与无法识别的输入返回 kind = 'unknown'。 */
export function detectQuery(raw: string): QueryDetection {
  const q = raw.trim()
  const unknown: QueryDetection = { kind: 'unknown', label: null, steamId64: null }
  if (!q) return unknown

  if (isSteam64(q)) return { kind: 'steamid64', label: 'SteamID64', steamId64: q }

  const steam2 = parseSteam2(q)
  if (steam2) return { kind: 'steam2', label: 'SteamID（STEAM_x:y:z）', steamId64: steam2 }

  const steam3 = parseSteam3(q)
  if (steam3) return { kind: 'steam3', label: 'SteamID3（[U:1:w]）', steamId64: steam3 }

  const account = parseAccountId(q)
  if (account) return { kind: 'account', label: 'SteamID3 数字账号', steamId64: account }

  const profile = parseProfileUrl(q)
  if (profile) {
    return profile.vanity
      ? { kind: 'vanity', label: 'Steam 自定义主页链接', steamId64: null }
      : { kind: 'profile', label: 'Steam 个人主页链接', steamId64: profile.value }
  }

  return unknown
}
