<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { NAlert, NAvatar, NInput, NPagination, NSelect, NSpin, NTag } from 'naive-ui'
import { RouterLink } from 'vue-router'
import RankBadge from '@/components/RankBadge.vue'
import { useApi } from '@/composables/useApi'
import { useLiveReload } from '@/composables/useLive'
import { MODES } from '@/types/common'
import type { Mode } from '@/types/common'
import { avatarUrl } from '@/utils/avatar'
import { formatDaysAgo, formatNumber } from '@/utils/format'
import { fetchLeaderboard } from './api'
import type { ActivityFilter, LeaderboardPage as LeaderboardPageData, SortKey } from './types'

/** 每列每页展示 50 人；总数 / 搜索 / 排序全部由服务端处理。 */
const PAGE_SIZE = 50

const query = ref('')
const activity = ref<ActivityFilter>('all')
const sortKey = ref<SortKey>('points')
const pages = ref<Record<Mode, number>>({ KZT: 1, SKZ: 1, VNL: 1 })

const { data, loading, error, reload } = useApi(async () => {
  const [kzt, skz, vnl] = await Promise.all(
    MODES.map((m) =>
      fetchLeaderboard(m, {
        page: pages.value[m],
        per_page: PAGE_SIZE,
        query: query.value.trim(),
        sort: sortKey.value,
        activity: activity.value,
      }),
    ),
  )
  return { KZT: kzt, SKZ: skz, VNL: vnl } as Record<Mode, LeaderboardPageData>
})

// 榜单每 10 分钟刷新一次，事件到达时自动更新
useLiveReload(() => ['leaderboard'], () => void reload())

const activityOptions: { label: string; value: ActivityFilter }[] = [
  { label: '全部活跃度', value: 'all' },
  { label: '近 7 天有游玩', value: '7' },
  { label: '近 30 天有游玩', value: '30' },
  { label: '超过 30 天未玩', value: 'inactive' },
]

const sortOptions: { label: string; value: SortKey }[] = [
  { label: '按总积分排序', value: 'points' },
  { label: '按完成地图排序', value: 'maps' },
  { label: '按平均分排序', value: 'avg' },
  { label: '按 Rating 排序', value: 'rating' },
]

function resetPages(): void {
  pages.value = { KZT: 1, SKZ: 1, VNL: 1 }
}

// 搜索防抖；筛选/排序变化立即重置页码并刷新
let searchTimer: ReturnType<typeof setTimeout> | undefined
watch(query, () => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    resetPages()
    void reload()
  }, 300)
})
watch([activity, sortKey], () => {
  resetPages()
  void reload()
})
onUnmounted(() => {
  if (searchTimer) clearTimeout(searchTimer)
})

function changePage(mode: Mode, page: number): void {
  if (pages.value[mode] === page) return
  pages.value = { ...pages.value, [mode]: page }
  void reload()
}

const columns = computed(() =>
  MODES.map((mode) => {
    const page = data.value?.[mode] ?? null
    return { mode, page, rows: page?.data ?? [], total: page?.pagination.total ?? 0 }
  }),
)

function pageCount(total: number): number {
  return Math.max(1, Math.ceil(total / PAGE_SIZE))
}

function rangeText(page: LeaderboardPageData): string {
  const from = (page.pagination.page - 1) * page.pagination.per_page + 1
  const to = Math.min(page.pagination.page * page.pagination.per_page, page.pagination.total)
  return `${from}–${to} / 共 ${page.pagination.total} 人`
}
</script>

<template>
  <div class="page-head page-head-row">
    <div>
      <h1 class="page-title">玩家排行榜</h1>
      <p class="page-sub">KZT / SKZ / VNL 三模式独立计分，同页并排展示</p>
    </div>
  </div>

  <div class="toolbar">
    <NInput v-model:value="query" placeholder="搜索玩家名称 / SteamID64..." clearable style="max-width: 320px" />
    <NSelect v-model:value="activity" :options="activityOptions" style="width: 180px" />
    <NSelect v-model:value="sortKey" :options="sortOptions" style="width: 180px" />
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>

    <div class="lb-columns">
      <div
        v-for="col in columns"
        :key="col.mode"
        class="panel lb-panel"
        :class="`mode-${col.mode.toLowerCase()}`"
      >
        <div class="panel-head">
          <div class="lb-head">
            <span class="mode-dot" />
            <span class="panel-title">{{ col.mode }}</span>
          </div>
          <div class="panel-head-extra">
            <NTag type="success" size="small">共 {{ col.total }} 人</NTag>
          </div>
        </div>

        <div class="lb-list">
          <div v-for="e in col.rows" :key="e.steam_id64" class="lb-row">
            <RankBadge :rank="e.rank" />
            <NAvatar :src="avatarUrl(e.steam_id64, e.avatar)" :size="28" round class="lb-avatar" />
            <div class="lb-main">
              <div class="lb-line">
                <RouterLink
                  class="lb-name"
                  :to="`/profile/${e.steam_id64}`"
                  :title="e.player"
                >
                  {{ e.player }}
                </RouterLink>
                <span class="lb-points mono">{{ formatNumber(e.total_points) }}</span>
              </div>
              <div class="lb-line lb-meta">
                <span>{{ e.maps_completed }} 图</span>
                <span class="sep">·</span>
                <span>{{ e.records }} 条</span>
                <span class="sep">·</span>
                <span>均 {{ e.avg_points }}</span>
                <span class="sep">·</span>
                <span>{{ formatDaysAgo(e.last_played_days_ago) }}</span>
              </div>
            </div>
          </div>

          <div v-if="!col.rows.length" class="lb-empty">
            {{ query ? '没有匹配的玩家' : (loading ? '同步中…' : '暂无榜单数据') }}
          </div>
        </div>

        <div v-if="col.page && col.page.pagination.total" class="lb-foot">
          <span class="lb-range">{{ rangeText(col.page) }}</span>
          <NPagination
            v-if="pageCount(col.total) > 1"
            size="small"
            :page="col.page.pagination.page"
            :page-count="pageCount(col.total)"
            @update:page="(p) => changePage(col.mode, p)"
          />
        </div>
      </div>
    </div>
  </NSpin>
</template>

<style scoped>
.lb-columns {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
  align-items: start;
}

/* 三列并排时取消全局「相邻面板上间距」，否则第 2/3 列会整体下沉 16px */
.lb-panel + .lb-panel {
  margin-top: 0;
}

/* 每列用对应模式色做顶部强调条 */
.lb-panel {
  border-top: 3px solid var(--mode-color, var(--accent));
}

.lb-head .panel-title {
  color: var(--mode-color, var(--accent));
}

.lb-head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mode-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--mode-color, var(--accent));
  box-shadow: 0 0 0 3px var(--mode-ring, var(--ring-accent));
}

.lb-list {
  display: flex;
  flex-direction: column;
}

.lb-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
}

.lb-row + .lb-row {
  border-top: 1px solid var(--border-color);
}

.lb-row:hover {
  background-color: var(--row-hover);
}

.lb-avatar {
  flex-shrink: 0;
  background-color: var(--bg-subtle);
}

.lb-main {
  flex: 1;
  min-width: 0;
}

.lb-line {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
}

.lb-line + .lb-line {
  margin-top: 2px;
}

.lb-name {
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--accent);
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.lb-name:hover {
  color: var(--accent-hover);
  text-decoration: underline;
  text-underline-offset: 2px;
}

.lb-points {
  flex-shrink: 0;
  font-size: 13px;
  font-weight: 700;
  color: var(--mode-color, var(--accent));
}

.lb-meta {
  flex-wrap: wrap;
  justify-content: flex-start;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
}

.lb-meta .sep {
  color: var(--border-strong);
}

.lb-empty {
  padding: 36px 16px;
  text-align: center;
  font-size: 13px;
  color: var(--text-muted);
}

.lb-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
  padding: 10px 16px;
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  color: var(--text-muted);
  background-color: var(--bg-subtle);
}

.lb-range {
  flex-shrink: 0;
}

.lb-foot :deep(.n-pagination) {
  flex-wrap: wrap;
}

@media (max-width: 1080px) {
  .lb-columns {
    grid-template-columns: 1fr;
  }
}
</style>
