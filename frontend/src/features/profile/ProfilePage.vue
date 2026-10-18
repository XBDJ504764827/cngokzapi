<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { NAlert, NAvatar, NButton, NCard, NDataTable, NInput, NSpin, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useApi } from '@/composables/useApi'
import { MODES } from '@/types/common'
import type { Mode } from '@/types/common'
import { avatarUrl } from '@/utils/avatar'
import { formatNumber } from '@/utils/format'
import { mapImage } from '@/utils/mapImage'
import { resolvePlayer } from '@/features/players/api'
import { fetchProfile } from './api'
import type { PlayerScoreEntry } from './types'

const route = useRoute()
const router = useRouter()

const steamId = computed(() => (route.params.steam_id as string | undefined) ?? '')
const input = ref(steamId.value)
const queryError = ref('')
const resolving = ref(false)

watch(steamId, (v) => (input.value = v))

const { data, loading, error, reload } = useApi(() => fetchProfile(steamId.value), false)

watch(
  steamId,
  (v) => {
    if (v) void reload()
  },
  { immediate: true },
)

// 首次访问会触发后台同步（PROJECT §9.5）：同步中自动重试
let timer: ReturnType<typeof setTimeout> | undefined
let retries = 0
watch(data, (d) => {
  if (timer) clearTimeout(timer)
  if (d?.syncing && retries < 6) {
    retries += 1
    timer = setTimeout(() => void reload(), 8000)
  }
})
watch(steamId, () => (retries = 0))
onUnmounted(() => {
  if (timer) clearTimeout(timer)
})

async function submit(): Promise<void> {
  const q = input.value.trim()
  if (!q || resolving.value) return
  queryError.value = ''
  resolving.value = true
  try {
    const { steam_id64 } = await resolvePlayer(q)
    void router.push(`/profile/${steam_id64}`)
  } catch (e) {
    queryError.value = e instanceof Error ? e.message : '查询失败，请检查输入'
  } finally {
    resolving.value = false
  }
}

const scores = computed(() => data.value?.scores ?? [])

const modeColumns: DataTableColumns<PlayerScoreEntry> = [
  { title: '模式', key: 'mode', width: 80 },
  { title: '全球排名', key: 'rank', width: 100, render: (r) => (r.rank === null ? '—' : `#${r.rank}`) },
  { title: '全球积分', key: 'points', width: 120, render: (r) => formatNumber(r.points) },
  {
    title: '本站已同步合计',
    key: 'points_local',
    width: 150,
    render: (r) => (r.points_local === null ? '—' : formatNumber(r.points_local)),
  },
  { title: '完成地图', key: 'maps_completed', width: 100 },
  { title: 'WR（NUB / PRO）', key: 'wrs', width: 140, render: (r) => `${r.wrs_nub} / ${r.wrs_pro}` },
  { title: '已同步成绩', key: 'records', width: 110 },
]

/** 最近成绩按模式分组（后端已按模式各取 20 条有分记录）。 */
const recentByMode = computed(() =>
  MODES.map((m: Mode) => ({
    mode: m,
    records: (data.value?.recent_records ?? []).filter((r) => r.mode === m),
  })),
)

/** 地图缩略图加载失败时隐藏，露出底色占位。 */
function onImageError(e: Event): void {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">个人资料</h1>
    <p class="page-sub">按 SteamID64 查询，首次查询会在后台从上游同步该玩家数据</p>
  </div>

  <NCard title="玩家查询" class="query-card">
    <div class="toolbar">
      <NInput
        v-model:value="input"
        placeholder="SteamID / SteamID64 / Steam 个人主页链接"
        style="max-width: 360px"
        @keyup.enter="submit"
      />
      <NButton type="primary" :loading="resolving" @click="submit">查询</NButton>
    </div>
    <NAlert v-if="queryError" type="error" style="margin-bottom: 12px">{{ queryError }}</NAlert>
    <NAlert v-if="!steamId" type="info">
      直接访问 <code>/profile/&lt;SteamID64&gt;</code>，或点击排行榜 / 封禁名单 / 地图榜中的玩家名进入。
    </NAlert>
  </NCard>

  <NSpin v-if="steamId" :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    <NAlert v-if="data?.syncing" type="info" style="margin-bottom: 12px">
      该玩家数据正在后台同步，页面稍后自动刷新（首次约需十几秒）。
    </NAlert>

    <div v-if="data" class="profile-hero">
      <NAvatar :src="avatarUrl(data.steam_id64, data.avatar)" :size="72" round class="profile-avatar" />
      <div class="profile-info">
        <div class="profile-name">{{ data.name || '未知玩家' }}</div>
        <div class="profile-meta">
          <span class="mono profile-steam">{{ data.steam_id64 }}</span>
          <span v-if="data.region" class="chip">{{ data.region }}</span>
        </div>
      </div>
    </div>

    <div v-if="scores.length" class="panel">
      <div class="panel-head">
        <div>
          <div class="panel-title">模式积分</div>
          <div class="panel-sub">全球积分为上游口径，本站合计仅含已同步成绩</div>
        </div>
      </div>
      <NDataTable :columns="modeColumns" :data="scores" :pagination="false" />
    </div>
    <NAlert v-else-if="data && !data.syncing" type="warning" style="margin-bottom: 12px">
      上游没有该玩家的榜单记录（可能未上榜或 SteamID64 有误）。
    </NAlert>

    <section class="section recent-section">
      <div class="section-header">
        <h2 class="section-title">最近成绩</h2>
        <span class="section-note">按模式分组 · 仅显示有分成绩 · 点击地图查看详情</span>
      </div>

      <div class="recent-columns">
        <div v-for="col in recentByMode" :key="col.mode" class="panel recent-col">
          <div class="panel-head">
            <div class="rec-head">
              <span class="mode-dot" />
              {{ col.mode }}
            </div>
            <div class="panel-head-extra">
              <NTag type="success" size="small">共 {{ col.records.length }} 条</NTag>
            </div>
          </div>

          <div class="rec-list">
            <div
              v-for="r in col.records"
              :key="`${r.map_id}-${r.stage}-${r.time}-${r.date}`"
              class="rec-row"
            >
              <RouterLink
                class="rec-thumb"
                :to="`/maps/${r.map_id}`"
                :title="`查看 ${r.map} 详情`"
              >
                <div class="rec-thumb-fallback">
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="M9 3 3 6v15l6-3 6 3 6-3V3l-6 3-6-3Zm0 0v15m6-12v15" />
                  </svg>
                </div>
                <img
                  :src="mapImage(r.map, 'thumbnails')"
                  :alt="r.map"
                  loading="lazy"
                  @error="onImageError"
                />
              </RouterLink>

              <div class="rec-body">
                <div class="rec-line">
                  <RouterLink class="rec-map" :to="`/maps/${r.map_id}`" :title="r.map">
                    {{ r.map }}
                  </RouterLink>
                  <span class="rec-time mono">{{ r.time }}</span>
                </div>
                <div class="rec-line rec-meta">
                  <span>{{ r.stage === 0 ? '主关' : `B${r.stage}` }}</span>
                  <span class="sep">·</span>
                  <span class="rec-points mono">{{ r.points }} 分</span>
                  <span class="sep">·</span>
                  <span class="mono">{{ r.date }}</span>
                </div>
              </div>
            </div>

            <div v-if="!col.records.length" class="rec-empty">暂无有分成绩</div>
          </div>
        </div>
      </div>
    </section>

    <div class="back">
      <RouterLink to="/leaderboard">← 返回排行榜</RouterLink>
    </div>
  </NSpin>
</template>

<style scoped>
.query-card {
  margin-bottom: 20px;
}

.profile-hero {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 18px 20px;
  margin-bottom: 16px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-sm);
}

.profile-avatar {
  border: 2px solid var(--accent-soft);
  background-color: var(--bg-subtle);
}

.profile-info {
  min-width: 0;
}

.profile-name {
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin-bottom: 6px;
}

.profile-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.profile-steam {
  font-size: 13px;
  color: var(--text-muted);
}

/* ---------- 最近成绩（三模式并排） ---------- */
.recent-section {
  margin-top: 26px;
}

.section-note {
  font-size: 12px;
  color: var(--text-muted);
}

.recent-columns {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
  align-items: start;
}

/* 同行面板取消全局「相邻面板上间距」，否则第 2/3 列会下沉 */
.recent-columns .panel + .panel {
  margin-top: 0;
}

.rec-head {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 700;
}

.mode-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(47, 111, 79, 0.14);
}

.rec-list {
  display: flex;
  flex-direction: column;
}

.rec-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
}

.rec-row + .rec-row {
  border-top: 1px solid var(--border-color);
}

.rec-row:hover {
  background-color: #f6f9f7;
}

.rec-thumb {
  position: relative;
  flex-shrink: 0;
  width: 72px;
  height: 42px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-subtle);
  overflow: hidden;
  transition:
    box-shadow 0.15s ease,
    transform 0.15s ease;
}

.rec-thumb:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(28, 28, 26, 0.16);
}

.rec-thumb-fallback {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.rec-thumb img {
  position: relative;
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.rec-body {
  flex: 1;
  min-width: 0;
}

.rec-line {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
}

.rec-line + .rec-line {
  margin-top: 2px;
}

.rec-map {
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.rec-map:hover {
  color: var(--accent);
  text-decoration: underline;
  text-underline-offset: 2px;
}

.rec-time {
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 700;
  color: var(--accent);
}

.rec-meta {
  justify-content: flex-start;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
}

.rec-meta .sep {
  color: var(--border-strong);
}

.rec-points {
  color: #8a6a2f;
  font-weight: 700;
}

.rec-empty {
  padding: 36px 16px;
  text-align: center;
  font-size: 13px;
  color: var(--text-muted);
}

.back {
  margin-top: 20px;
}

.back a {
  color: var(--accent);
  text-decoration: none;
  font-weight: 600;
}

@media (max-width: 1080px) {
  .recent-columns {
    grid-template-columns: 1fr;
  }

  .recent-columns .panel + .panel {
    margin-top: 16px;
  }
}
</style>
