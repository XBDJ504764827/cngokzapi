// 纯展示函数：与后端 mock.rs 的口径一致（今天/昨天/N天前）。

export function formatDaysAgo(days: number): string {
  if (days <= 0) return '今天'
  if (days === 1) return '昨天'
  return `${days}天前`
}

export function formatNumber(n: number): string {
  return n.toLocaleString()
}
