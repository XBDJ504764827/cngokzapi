<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
import { NAlert, NAvatar, NDataTable, NInput, NPagination, NSpin, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { useApi } from '@/composables/useApi'
import { fetchBans } from './api'
import type { BanEntry } from './types'

const PAGE_SIZE = 20

const query = ref('')
const page = ref(1)
const { data, loading, error, reload } = useApi(() => fetchBans(query.value, page.value))

watch(query, () => {
  if (page.value === 1) {
    void reload()
  } else {
    page.value = 1
  }
})
watch(page, () => void reload())

const rows = computed<BanEntry[]>(() => data.value?.data ?? [])
const total = computed(() => data.value?.pagination.total ?? 0)
const pageCount = computed(() => Math.max(1, Math.ceil(total.value / PAGE_SIZE)))
const countText = computed(() =>
  query.value.trim() === '' ? `共 ${total.value} 人` : `筛出 ${total.value} 人`,
)

const columns: DataTableColumns<BanEntry> = [
  {
    title: '玩家',
    key: 'player',
    render: (row) =>
      h('div', { class: 'lb-player' }, [
        h(NAvatar, { src: row.avatar, size: 32, round: true }),
        h('div', { class: 'lb-id' }, [
          h('div', { class: 'lb-name' }, row.player),
          h('div', { class: 'lb-steam' }, row.steam_id64),
        ]),
      ]),
  },
  {
    title: '封禁理由',
    key: 'reason',
    width: 150,
    render: (row) => h('span', { class: 'ban-reason' }, row.reason),
  },
  { title: '封禁时间', key: 'banned_at', width: 120 },
  { title: '封禁服务器', key: 'server', width: 220 },
  {
    title: '详细数据',
    key: 'detail',
    render: (row) => h('span', { class: 'ban-detail' }, row.detail),
  },
]
</script>

<template>
  <div class="page-head leaderboard-head">
    <div>
      <h1 class="page-title">封禁名单</h1>
      <p class="page-sub">作弊玩家公示：历史成绩保留，但不再参与排名</p>
    </div>
    <NTag type="success" size="medium">{{ countText }}</NTag>
  </div>

  <div class="toolbar">
    <NInput v-model:value="query" placeholder="搜索玩家名称 / SteamID64..." clearable style="max-width: 300px" />
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    <NDataTable :columns="columns" :data="rows" :pagination="false" />
    <div v-if="pageCount > 1" class="pager">
      <NPagination v-model:page="page" :page-count="pageCount" :page-size="PAGE_SIZE" :page-sizes="[]" />
      <span class="page-info">第 {{ page }} / {{ pageCount }} 页</span>
    </div>
  </NSpin>
</template>

<style scoped>
.leaderboard-head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
}

.lb-player {
  display: flex;
  align-items: center;
  gap: 10px;
}

.lb-name {
  font-weight: 600;
  line-height: 1.4;
}

.lb-steam {
  font-size: 12px;
  color: var(--text-muted);
  font-family: 'SF Mono', 'Courier New', monospace;
}

.ban-reason {
  font-weight: 600;
  color: var(--danger);
}

.ban-detail {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: 'SF Mono', 'Courier New', monospace;
}

.pager {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  margin-top: 16px;
}

.page-info {
  font-size: 13px;
  color: var(--text-muted);
}
</style>
