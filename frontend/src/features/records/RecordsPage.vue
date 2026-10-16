<script setup lang="ts">
import { computed } from 'vue'
import { NAlert, NCard, NDataTable, NSpin, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { useApi } from '@/composables/useApi'
import { MODES } from '@/types/common'
import type { Mode } from '@/types/common'
import { fetchRecords } from './api'
import type { RecordEntry } from './types'

const { data, loading, error } = useApi(fetchRecords)

const columns: DataTableColumns<RecordEntry> = [
  { title: '玩家', key: 'player' },
  { title: '地图', key: 'map' },
  { title: '时间', key: 'time' },
  { title: '分数', key: 'points' },
  { title: '日期', key: 'date' },
]

function rowsFor(mode: Mode): RecordEntry[] {
  return (data.value ?? []).filter((r) => r.mode === mode).slice(0, 20)
}

const hasData = computed(() => (data.value ?? []).length > 0)
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">最新记录</h1>
    <p class="page-sub">三模式并排，各列最新 20 条</p>
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    <div v-if="hasData" class="records-columns">
      <NCard v-for="mode in MODES" :key="mode" :title="mode" size="small">
        <template #header-extra>
          <NTag type="success" size="small">最新 20 条</NTag>
        </template>
        <NDataTable :columns="columns" :data="rowsFor(mode)" :pagination="false" size="small" />
      </NCard>
    </div>
  </NSpin>
</template>

<style scoped>
.records-columns {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}

@media (max-width: 1080px) {
  .records-columns {
    grid-template-columns: 1fr;
  }
}
</style>
