// 头像地址：优先用已同步的真实头像；没有则用按 SteamID64 生成的确定性占位图
// （与重构前一致的外观，避免"一片灰圆圈"）。真实头像的懒加载见 PROJECT §10 下一步。

/** DiceBear identicon：同一个 SteamID64 永远得到同一张图。 */
export function fallbackAvatar(seed: string): string {
  return `https://api.dicebear.com/7.x/identicon/svg?seed=${encodeURIComponent(seed)}`
}

/** 有真实头像用真实的，否则用占位图。 */
export function avatarUrl(steamId64: string, avatar?: string | null): string {
  return avatar && avatar.length > 0 ? avatar : fallbackAvatar(steamId64)
}
