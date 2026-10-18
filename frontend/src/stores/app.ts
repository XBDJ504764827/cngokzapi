// 全局状态。全站不再提供模式切换（2026-10-18）：
// 排行榜同时展示 KZT/SKZ/VNL 三模式；地图 / 地图详情默认按 KZT 展示。
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Mode } from '@/types/common'

export const useAppStore = defineStore('app', () => {
  const currentMode = ref<Mode>('KZT')

  return { currentMode }
})
