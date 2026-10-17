// 地图图片地址：使用 KZGlobalTeam 公开地图图库（GitHub raw），
// 传地图名即可；上游没有该图时页面自行降级为占位块（见各页 @error 处理）。
// 后续若要换成自建图床/CDN，只需改这里（PROJECT §10 下一步）。

const BASE = 'https://raw.githubusercontent.com/KZGlobalTeam/map-images/public'

export type MapImageSize = 'mediums' | 'thumbnails'

/** `mediums` 用于卡片大图（约 12-25KB），`thumbnails` 用于列表小图（约 2-5KB）。 */
export function mapImage(name: string, size: MapImageSize = 'mediums'): string {
  if (!name) return ''
  return `${BASE}/${size}/${encodeURIComponent(name)}.jpg`
}
