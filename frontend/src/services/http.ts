// API 请求封装：全站唯一 fetch 出口，统一错误体 { error: { code, message } }。
import { ApiError } from '@/types/common'

const BASE = '/api/v1'

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(BASE + path, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  })
  if (!res.ok) {
    let code = 'UNKNOWN'
    let message = `请求失败 (${res.status})`
    try {
      const body = await res.json()
      code = body.error?.code ?? code
      message = body.error?.message ?? message
    } catch {
      // 非 JSON 错误体，保持默认文案
    }
    throw new ApiError(message, code, res.status)
  }
  return (await res.json()) as T
}

export const http = {
  get<T>(path: string): Promise<T> {
    return request<T>(path)
  },
}
