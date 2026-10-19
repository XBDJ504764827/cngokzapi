// SSE 实时订阅：页面只订阅自己关心的主题，收到事件后防抖刷新。
// 后端端点：GET /api/v1/stream?topics=records,map:44,leaderboard
import { onUnmounted, watch } from 'vue'

export interface LiveEvent {
  topic: string
  kind: string
  id?: string
  at: string
}

/**
 * 订阅实时事件并调用 `reload`。
 * - `topics` 返回空数组时订阅全部事件；
 * - 默认 400ms 防抖，避免一批事件触发多次请求；
 * - EventSource 断线由浏览器自动重连。
 */
export function useLiveReload(
  topics: () => string[],
  reload: () => void,
  delay = 400,
): void {
  let source: EventSource | null = null
  let timer: ReturnType<typeof setTimeout> | undefined

  function schedule(): void {
    if (timer) return
    timer = setTimeout(() => {
      timer = undefined
      reload()
    }, delay)
  }

  function close(): void {
    source?.close()
    source = null
  }

  function connect(): void {
    close()
    const list = topics().filter((t) => t.length > 0)
    const url = list.length
      ? `/api/v1/stream?topics=${encodeURIComponent(list.join(','))}`
      : '/api/v1/stream'
    source = new EventSource(url)
    source.onmessage = () => schedule()
  }

  watch(() => topics().join(','), connect, { immediate: true })

  onUnmounted(() => {
    close()
    if (timer) clearTimeout(timer)
  })
}
