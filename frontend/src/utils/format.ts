// 纯展示函数。

/** 距今天数 → 文案；null 表示本地尚未同步到该玩家成绩。 */
export function formatDaysAgo(days: number | null): string {
  if (days === null || days < 0) return '—'
  if (days === 0) return '今天'
  if (days === 1) return '昨天'
  return `${days}天前`
}

export function formatNumber(n: number): string {
  return n.toLocaleString()
}

/** 文件大小：B → MB（保留一位小数）。 */
export function formatFilesize(bytes: number): string {
  if (bytes <= 0) return '—'
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

/** 三模式 tier 展示；0 表示该模式未评级。 */
export function formatTier(tier: number): string {
  return tier > 0 ? `T${tier}` : '—'
}

/** UTC ISO8601 → `MM-DD HH:mm`（本地时区）；无效时原样返回。 */
export function formatDateTimeShort(iso: string): string {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  const p = (n: number) => String(n).padStart(2, '0')
  return `${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}

/** UTC ISO8601 → `YYYY-MM-DD HH:mm`（本地时区，带年份）；无效时原样返回。 */
export function formatDateTime(iso: string): string {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}

/** UTC ISO8601 → `YY-MM-DD HH:mm`（本地时区，紧凑带年份）；无效时原样返回。 */
export function formatDateTimeTiny(iso: string): string {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  const p = (n: number) => String(n).padStart(2, '0')
  return `${String(d.getFullYear()).slice(2)}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}
