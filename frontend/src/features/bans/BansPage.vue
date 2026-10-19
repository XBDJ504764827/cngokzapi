<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
import { NAlert, NDataTable, NEmpty, NInput, NPagination, NSpin, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import PlayerCell from '@/components/PlayerCell.vue'
import { useApi } from '@/composables/useApi'
import { useLiveReload } from '@/composables/useLive'
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

// 新封禁到达时自动刷新
useLiveReload(() => ['bans'], () => void reload())

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
    width: 240,
    render: (row) =>
      h(PlayerCell, {
        player: row.player,
        steamId64: row.steam_id64,
        avatar: row.avatar,
      }),
  },
  {
    title: '封禁理由',
    key: 'reason',
    width: 150,
    render: (row) => h('span', { class: 'ban-reason' }, row.reason),
  },
  { title: '封禁时间', key: 'banned_at', width: 120, render: (row) => h('span', { class: 'mono' }, row.banned_at) },
  { title: '封禁服务器', key: 'server', width: 220, ellipsis: { tooltip: true } },
  {
    title: '详细数据',
    key: 'detail',
    render: (row) => h('span', { class: 'ban-detail' }, row.detail),
  },
]
</script>

<template>
  <div class="page-head page-head-row">
    <div>
      <h1 class="page-title">封禁名单</h1>
      <p class="page-sub">作弊玩家公示：历史成绩保留，但不再参与排名</p>
    </div>
    <div class="page-head-actions">
      <NTag type="success" size="medium">{{ countText }}</NTag>
    </div>
  </div>

  <div class="toolbar">
    <NInput v-model:value="query" placeholder="搜索玩家名称 / SteamID64..." clearable style="max-width: 320px" />
  </div>

  <div class="panel">
    <div class="panel-head">
      <div>
        <div class="panel-title">封禁记录</div>
        <div class="panel-sub">数据来自上游封禁同步 · 每页 {{ PAGE_SIZE }} 条</div>
      </div>
    </div>

    <NSpin :show="loading">
      <NAlert v-if="error" type="error" style="margin: 16px 20px 0">{{ error }}</NAlert>
      <NEmpty
        v-else-if="!loading && rows.length === 0"
        description="暂无封禁记录"
        style="padding: 48px 0"
      />
      <NDataTable v-else :columns="columns" :data="rows" :pagination="false" />
    </NSpin>
  </div>

  <div v-if="pageCount > 1" class="pager">
    <NPagination v-model:page="page" :page-count="pageCount" :page-size="PAGE_SIZE" :page-sizes="[]" />
    <span class="page-info">第 {{ page }} / {{ pageCount }} 页</span>
  </div>
</template>

<style scoped>
:deep(.ban-reason) {
  font-weight: 600;
  color: var(--danger);
}

:deep(.ban-detail) {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
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
