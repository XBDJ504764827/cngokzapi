// 国家/地区代码展示：国旗图片 + 中文名（个人资料页用）。
// 国旗用 flagcdn 图片而非 emoji（Windows 不渲染 emoji 国旗），缺图时只显示文字。

/** ISO 3166 alpha-2 → 国旗图片地址；非法代码返回 null。 */
export function flagUrl(code: string | null | undefined): string | null {
  if (!code || !/^[a-z]{2}$/i.test(code)) return null
  return `https://flagcdn.com/24x18/${code.toLowerCase()}.png`
}

/** ISO 3166 alpha-2 → 中文国家/地区名；无法识别时返回大写代码。 */
export function countryName(code: string | null | undefined): string | null {
  if (!code || !/^[a-z]{2}$/i.test(code)) return null
  const upper = code.toUpperCase()
  try {
    return new Intl.DisplayNames(['zh-CN'], { type: 'region' }).of(upper) ?? upper
  } catch {
    return upper
  }
}
