// 页面数据加载范式：{ data, loading, error, reload }，各页统一用法。
import { ref } from 'vue'

export function useApi<T>(loader: () => Promise<T>, immediate = true) {
  const data = ref<T | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function reload(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      data.value = await loader()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载失败'
    } finally {
      loading.value = false
    }
  }

  if (immediate) {
    void reload()
  }

  return { data, loading, error, reload }
}
