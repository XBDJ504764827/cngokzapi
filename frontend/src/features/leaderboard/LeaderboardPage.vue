<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
import { NAlert, NDataTable, NEmpty, NInput, NSelect, NSpin, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import PlayerCell from '@/components/PlayerCell.vue'
import RankBadge from '@/components/RankBadge.vue'
import { useApi } from '@/composables/useApi'
import { useAppStore } from '@/stores/app'
import { formatDaysAgo, formatNumber } from '@/utils/format'
import { fetchLeaderboard } from './api'
import type { ActivityFilter, LeaderboardEntry, SortKey } from './types'

const app = useAppStore()
const { data, loading, error, reload } = useApi(() => fetchLeaderboard(app.currentMode))
watch(
  () => app.currentMode,
  () => void reload(),
)

const query = ref('')
const activity = ref<ActivityFilter>('all')
const sortKey = ref<SortKey>('total_points')

const activityOptions: { label: string; value: ActivityFilter }[] = [
  { label: '全部活跃度', value: 'all' },
  { label: '近 7 天有游玩', value: '7' },
  { label: '近 30 天有游玩', value: '30' },
  { label: '超过 30 天未玩', value: 'inactive' },
]

const sortOptions: { label: string; value: SortKey }[] = [
  { label: '按总分数排序', value: 'total_points' },
  { label: '按完成地图排序', value: 'maps_completed' },
  { label: '按平均分排序', value: 'avg_points' },
  { label: '按记录数排序', value: 'records' },
]

// 活跃度筛选只对"本地已同步到成绩"的玩家有意义；未同步（null）一律不参与活跃度命中
const rows = computed<LeaderboardEntry[]>(() => {
  const all = data.value ?? []
  const q = query.value.trim().toLowerCase()
  const filtered = all.filter((e) => {
    if (q && !e.player.toLowerCase().includes(q) && !e.steam_id64.includes(q)) return false
    const days = e.last_played_days_ago
    if (activity.value === '7' && (days === null || days > 7)) return false
    if (activity.value === '30' && (days === null || days > 30)) return false
    if (activity.value === 'inactive' && (days === null || days <= 30)) return false
    return true
  })
  return [...filtered].sort((a, b) => b[sortKey.value] - a[sortKey.value])
})

const countText = computed(() => {
  const total = data.value?.length ?? 0
  return rows.value.length === total
    ? `共 ${total} 人`
    : `筛出 ${rows.value.length} 人 / 共 ${total} 人`
})

const columns: DataTableColumns<LeaderboardEntry> = [
  {
    title: '排名',
    key: 'rank',
    width: 80,
    render: (row) => h(RankBadge, { rank: row.rank }),
  },
  {
    title: '玩家',
    key: 'player',
    render: (row) =>
      h(PlayerCell, {
        player: row.player,
        steamId64: row.steam_id64,
        avatar: row.avatar,
      }),
  },
  { title: '完成地图', key: 'maps_completed', width: 100 },
  {
    title: '总分数',
    key: 'total_points',
    width: 110,
    render: (row) => h('span', { class: 'mono points-strong' }, formatNumber(row.total_points)),
  },
  { title: '平均分', key: 'avg_points', width: 90, render: (row) => h('span', { class: 'mono' }, row.avg_points) },
  { title: '记录数', key: 'records', width: 90 },
  {
    title: '最近游玩',
    key: 'last_played',
    width: 120,
    render: (row) =>
      h('div', { class: 'lb-last' }, [
        h('div', { class: 'lb-active' }, formatDaysAgo(row.last_played_days_ago)),
        h('div', { class: 'lb-date' }, row.last_played ?? '未同步'),
      ]),
  },
]
</script>

<template>
  <div class="page-head page-head-row">
    <div>
      <h1 class="page-title">玩家排行榜</h1>
      <p class="page-sub">当前模式：{{ app.currentMode }} · 点击导航栏模式按钮切换</p>
    </div>
    <div class="page-head-actions">
      <NTag type="success" size="medium">{{ countText }}</NTag>
    </div>
  </div>

  <div class="toolbar">
    <NInput v-model:value="query" placeholder="搜索玩家名称 / SteamID64..." clearable style="max-width: 300px" />
    <NSelect v-model:value="activity" :options="activityOptions" style="width: 180px" />
    <NSelect v-model:value="sortKey" :options="sortOptions" style="width: 180px" />
  </div>

  <div class="panel">
    <div class="panel-head">
      <div>
        <div class="panel-title">{{ app.currentMode }} · 全球玩家排名</div>
        <div class="panel-sub">按总分排序 · 名次由本站汇总分计算</div>
      </div>
      <div class="panel-head-extra">
        <NTag v-if="loading" size="small">同步中…</NTag>
      </div>
    </div>

    <NSpin :show="loading">
      <NAlert v-if="error" type="error" style="margin: 16px 20px 0">{{ error }}</NAlert>
      <NEmpty
        v-else-if="!loading && (data?.length ?? 0) === 0"
        description="榜单正在从上游同步，稍后刷新即可"
        style="padding: 48px 0"
      />
      <NDataTable
        v-else
        :columns="columns"
        :data="rows"
        :pagination="{ pageSize: 20 }"
        :scroll-x="860"
      />
    </NSpin>
  </div>
</template>

<style scoped>
:deep(.points-strong) {
  font-weight: 700;
  color: var(--accent);
}

:deep(.lb-active) {
  font-weight: 600;
}

:deep(.lb-date) {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
