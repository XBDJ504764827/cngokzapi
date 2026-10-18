<script setup lang="ts">
import { computed, h } from 'vue'
import { NAlert, NAvatar, NCard, NDataTable, NEmpty, NSpin, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RouterLink } from 'vue-router'
import { useApi } from '@/composables/useApi'
import { MODES } from '@/types/common'
import type { Mode } from '@/types/common'
import { avatarUrl } from '@/utils/avatar'
import { fetchRecords } from './api'
import type { RecordEntry } from './types'

const { data, loading, error } = useApi(fetchRecords)

// 列宽固定 + 省略号（hover 出完整内容），确保任何长度都不会换行
const columns: DataTableColumns<RecordEntry> = [
  {
    title: '玩家',
    key: 'player',
    width: 132,
    render: (row) =>
      h('div', { class: 'player-inline' }, [
        h(NAvatar, { src: avatarUrl(row.steam_id64), size: 22, round: true }),
        h(
          RouterLink,
          { to: `/profile/${row.steam_id64}`, class: 'player-link', title: row.player },
          { default: () => row.player },
        ),
      ]),
  },
  { title: '地图', key: 'map', width: 130, ellipsis: { tooltip: true } },
  { title: '时间', key: 'time', width: 78, render: (row) => h('span', { class: 'time-value' }, row.time) },
  { title: '分数', key: 'points', width: 58, render: (row) => h('span', { class: 'mono' }, row.points) },
  { title: '日期', key: 'date', width: 88 },
]

/** 单列总宽（玩家/地图/时间/分数/日期），窄列时横向滚动而不换行。 */
const TABLE_WIDTH = 486

function rowsFor(mode: Mode): RecordEntry[] {
  return (data.value ?? []).filter((r) => r.mode === mode).slice(0, 20)
}

const hasData = computed(() => (data.value ?? []).length > 0)
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">最新记录</h1>
    <p class="page-sub">三模式并排，各列最新 20 条 · 点击玩家名查看资料</p>
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    <NEmpty v-else-if="!loading && !hasData" description="暂无记录数据" style="padding: 48px 0" />

    <div v-else class="records-columns">
      <NCard v-for="mode in MODES" :key="mode" size="small">
        <template #header>
          <span class="records-col-title">
            <span class="mode-dot" />
            {{ mode }}
          </span>
        </template>
        <template #header-extra>
          <NTag type="success" size="small">最新 20 条</NTag>
        </template>
        <NDataTable
          :columns="columns"
          :data="rowsFor(mode)"
          :pagination="false"
          :scroll-x="TABLE_WIDTH"
          size="small"
        />
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

.records-col-title {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.mode-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(47, 111, 79, 0.14);
}

:deep(.player-inline) {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

:deep(.player-link) {
  color: var(--text-primary);
  text-decoration: none;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

:deep(.player-link:hover) {
  color: var(--accent);
}

:deep(.time-value) {
  font-family: var(--font-mono);
  font-weight: 600;
  color: var(--accent);
}

/* 单元格一律不换行，超出用省略号（配合 ellipsis 列配置） */
:deep(.n-data-table-td),
:deep(.n-data-table-th) {
  white-space: nowrap;
}

@media (max-width: 1080px) {
  .records-columns {
    grid-template-columns: 1fr;
  }
}
</style>
