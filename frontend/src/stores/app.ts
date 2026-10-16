// 全局状态：当前模式（KZT/SKZ/VNL）是全站总开关，各榜单页订阅它。
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { MODES } from '@/types/common'
import type { Mode } from '@/types/common'

export const useAppStore = defineStore('app', () => {
  const currentMode = ref<Mode>('KZT')

  function setMode(mode: Mode): void {
    currentMode.value = mode
  }

  function cycleMode(): void {
    const i = MODES.indexOf(currentMode.value)
    currentMode.value = MODES[(i + 1) % MODES.length] as Mode
  }

  return { currentMode, setMode, cycleMode }
})
