// 个人资料等级：与 gokz.top 口径一致 —— 等级由 Rating（Rt）向下取整决定。
// 列表：New(1) Beginner(2) Amateur(3) Casual(4) Regular(5)
//       Skilled(6) Expert(7) Pro(8) Master(9) Legend(10)
// KZT / SKZ / VNL 三模式共用同一张等级表。

export const LEVEL_NAMES = [
  'New',
  'Beginner',
  'Amateur',
  'Casual',
  'Regular',
  'Skilled',
  'Expert',
  'Pro',
  'Master',
  'Legend',
] as const

export interface ModeLevel {
  /** 等级名，如 Legend */
  name: string
  /** 1（New）～10（Legend），用于等级条填充 */
  index: number
  /** 下一级名称；已是 Legend 时为 null */
  next: string | null
  /** 距离下一级的 Rating 差值；已是 Legend（或无法计算）时为 null */
  toNext: number | null
}

/** Rating → 等级；rating 为空时返回 null（展示为「未评级」）。 */
export function levelOf(rating: number | null | undefined): ModeLevel | null {
  // rating < 1 视为未评级（GOKZ 的 New 起点为 Rt 1.00）
  if (rating === null || rating === undefined || !Number.isFinite(rating) || rating < 1) {
    return null
  }
  const index = Math.min(LEVEL_NAMES.length, Math.max(1, Math.floor(rating)))
  const next = index < LEVEL_NAMES.length ? LEVEL_NAMES[index] : null
  const toNext = index < LEVEL_NAMES.length ? index + 1 - rating : null
  return { name: LEVEL_NAMES[index - 1], index, next, toNext }
}
