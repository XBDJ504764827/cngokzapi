<script setup lang="ts">
import { computed, h, onUnmounted, ref, watch } from 'vue'
import { NAlert, NDataTable, NSpin, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RouterLink, useRoute } from 'vue-router'
import PlayerCell from '@/components/PlayerCell.vue'
import RankBadge from '@/components/RankBadge.vue'
import { useApi } from '@/composables/useApi'
import { MODES } from '@/types/common'
import type { Mode } from '@/types/common'
import { formatDateTimeTiny, formatFilesize, formatTier } from '@/utils/format'
import { mapImage } from '@/utils/mapImage'
import { fetchMapDetail } from './api'
import type { MapRecordEntry } from './types'

const route = useRoute()
const mapId = computed(() => Number(route.params.id))
const { data, loading, error, reload } = useApi(() => fetchMapDetail(mapId.value))
watch(mapId, () => void reload())

// 首次访问会触发三模式整图同步（PROJECT §9.5）：同步完成前自动重试
let timer: ReturnType<typeof setTimeout> | undefined
let retries = 0
watch(data, (d) => {
  if (timer) clearTimeout(timer)
  if (d?.syncing && retries < 8) {
    retries += 1
    timer = setTimeout(() => void reload(), 10000)
  }
})
watch(mapId, () => (retries = 0))
onUnmounted(() => {
  if (timer) clearTimeout(timer)
})

const map = computed(() => data.value?.map ?? null)
const authors = computed(() =>
  map.value && map.value.mappers.length > 0 ? map.value.mappers.join(', ') : '作者未知',
)

const activeMode = ref<Mode>('KZT')
const activeBoard = computed(() => {
  const boards = data.value?.boards ?? []
  return boards.find((b) => b.mode === activeMode.value) ?? boards[0] ?? null
})

/** 关卡图加载失败时隐藏，露出底色占位。 */
function onImageError(e: Event): void {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

function tierClass(tier: number): string {
  return tier > 0 ? `tier-${tier}` : 'tier-0'
}

/** PRO 榜不放「存读点」列；TP 榜追加存读点次数。 */
function boardColumns(withTeleports: boolean): DataTableColumns<MapRecordEntry> {
  const cols: DataTableColumns<MapRecordEntry> = [
    {
      title: '排名',
      key: 'rank',
      width: 56,
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
          compact: true,
        }),
    },
    {
      title: '完成时间',
      key: 'time',
      width: 84,
      render: (row) => h('span', { class: 'mono time-strong' }, row.time),
    },
    {
      title: '分数',
      key: 'points',
      width: 52,
      render: (row) => h('span', { class: 'mono' }, row.points > 0 ? row.points : '—'),
    },
  ]
  if (withTeleports) {
    cols.push({
      title: '存读点',
      key: 'teleports',
      width: 58,
      render: (row) => h('span', { class: 'mono' }, `${row.teleports} 次`),
    })
  }
  cols.push({
    title: '上传时间',
    key: 'created_at',
    width: 126,
    render: (row) => h('span', { class: 'mono upload' }, formatDateTimeTiny(row.created_at)),
  })
  return cols
}

const proColumns = boardColumns(false)
const tpColumns = boardColumns(true)
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">{{ map?.name ?? '地图详情' }}</h1>
    <p class="page-sub">
      <RouterLink to="/maps">← 返回地图列表</RouterLink>
      · 主关 PRO / TP 排行榜 · 三模式可切换
    </p>
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    <NAlert v-if="data?.syncing" type="info" style="margin-bottom: 12px">
      该地图部分模式的完整成绩正在后台同步（首次约十几秒到一分钟），页面会自动刷新。
    </NAlert>

    <!-- 地图大图 + 基本信息 -->
    <div v-if="map" class="map-hero panel">
      <div class="hero-thumb">
        <div class="hero-fallback">
          <svg
            width="26"
            height="26"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M9 3 3 6v15l6-3 6 3 6-3V3l-6 3-6-3Zm0 0v15m6-12v15" />
          </svg>
          <span>{{ map.name }}</span>
        </div>
        <img :src="mapImage(map.name)" :alt="map.name" @error="onImageError" />
      </div>

      <div class="hero-info">
        <div class="hero-title-row">
          <h2 class="map-name">{{ map.name }}</h2>
          <a
            v-if="map.workshop_id"
            class="accent-link mono workshop"
            :href="`https://steamcommunity.com/sharedfiles/filedetails/?id=${map.workshop_id}`"
            target="_blank"
            rel="noreferrer"
          >
            创意工坊 {{ map.workshop_id }}
          </a>
        </div>

        <div class="tier-row">
          <span class="tier-chip" :class="tierClass(map.tier_kzt)">
            KZT <b>{{ formatTier(map.tier_kzt) }}</b>
          </span>
          <span class="tier-chip" :class="tierClass(map.tier_skz)">
            SKZ <b>{{ formatTier(map.tier_skz) }}</b>
          </span>
          <span class="tier-chip" :class="tierClass(map.tier_vnl)">
            VNL <b>{{ formatTier(map.tier_vnl) }}</b>
          </span>
        </div>

        <div class="hero-meta">
          <div class="meta-item">
            <span class="meta-label">制作人</span>
            <span class="meta-value" :title="authors">{{ authors }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">文件大小</span>
            <span class="meta-value">{{ formatFilesize(map.filesize) }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">奖励关</span>
            <span class="meta-value">Bonus ×{{ map.bonuses }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 三模式最快时间（PRO / TP 同时展示） -->
    <section v-if="data?.boards?.length" class="section">
      <div class="section-header">
        <h2 class="section-title">三模式最快完成</h2>
        <span class="section-count">PRO = 无存读点 · TP = 含存读点</span>
      </div>
      <div class="wr-summary">
        <div v-for="b in data.boards" :key="b.mode" class="wr-mode panel">
          <div class="wr-mode-head">
            <span class="mode-dot" />
            {{ b.mode }}
          </div>
          <div class="wr-item">
            <span class="wr-kind">PRO</span>
            <template v-if="b.wr_pro">
              <span class="wr-time mono">{{ b.wr_pro.time }}</span>
              <RouterLink class="wr-player" :to="`/profile/${b.wr_pro.steam_id64}`" :title="b.wr_pro.player">
                {{ b.wr_pro.player }}
              </RouterLink>
            </template>
            <span v-else class="wr-none">暂无成绩</span>
          </div>
          <div class="wr-item">
            <span class="wr-kind">TP</span>
            <template v-if="b.wr_tp">
              <span class="wr-time mono">{{ b.wr_tp.time }}</span>
              <RouterLink class="wr-player" :to="`/profile/${b.wr_tp.steam_id64}`" :title="b.wr_tp.player">
                {{ b.wr_tp.player }}
              </RouterLink>
            </template>
            <span v-else class="wr-none">暂无成绩</span>
          </div>
        </div>
      </div>
    </section>

    <!-- 模式切换（页内，仅影响下方详细榜单） -->
    <div class="mode-tabs">
      <button
        v-for="m in MODES"
        :key="m"
        class="mode-tab"
        :class="{ active: activeMode === m }"
        @click="activeMode = m"
      >
        <span class="mode-dot" />
        {{ m }}
      </button>
    </div>

    <!-- PRO / TP 详细榜 -->
    <div v-if="activeBoard" class="boards-grid">
      <div class="panel board">
        <div class="panel-head">
          <div>
            <div class="panel-title">PRO 排行榜</div>
            <div class="panel-sub">无存读点 · 主关前 100 名</div>
          </div>
          <div class="panel-head-extra">
            <NTag type="success" size="small">共 {{ activeBoard.pro.length }} 人</NTag>
          </div>
        </div>
        <NDataTable
          v-if="activeBoard.pro.length"
          :columns="proColumns"
          :data="activeBoard.pro"
          :pagination="{ pageSize: 20 }"
          size="small"
        />
        <div v-else class="board-empty">该模式暂无 PRO 成绩</div>
      </div>

      <div class="panel board">
        <div class="panel-head">
          <div>
            <div class="panel-title">TP 排行榜</div>
            <div class="panel-sub">含存读点 · 主关前 100 名</div>
          </div>
          <div class="panel-head-extra">
            <NTag type="success" size="small">共 {{ activeBoard.tp.length }} 人</NTag>
          </div>
        </div>
        <NDataTable
          v-if="activeBoard.tp.length"
          :columns="tpColumns"
          :data="activeBoard.tp"
          :pagination="{ pageSize: 20 }"
          size="small"
        />
        <div v-else class="board-empty">该模式暂无 TP 成绩</div>
      </div>
    </div>
  </NSpin>
</template>

<style scoped>
/* ---------- 地图大图 + 信息 ---------- */
.map-hero {
  display: grid;
  grid-template-columns: 440px minmax(0, 1fr);
  gap: 22px;
  padding: 18px;
  margin-bottom: 26px;
}

.hero-thumb {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  border-radius: var(--radius-md);
  background-color: var(--bg-subtle);
  overflow: hidden;
}

.hero-fallback {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 0 20px;
  color: var(--text-muted);
  font-size: 13px;
}

.hero-fallback span {
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hero-thumb img {
  position: relative;
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.hero-info {
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 14px;
}

.hero-title-row {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 12px;
}

.map-name {
  font-size: 26px;
  font-weight: 700;
  letter-spacing: -0.02em;
  word-break: break-all;
}

.workshop {
  font-size: 12px;
}

.tier-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tier-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-subtle);
  font-size: 12px;
  color: var(--text-secondary);
}

.tier-chip b {
  font-weight: 700;
}

.tier-0 {
  background-color: var(--bg-subtle);
  color: var(--text-muted);
}

.tier-1,
.tier-2 {
  background-color: var(--tier-easy);
  color: #fff;
}

.tier-3,
.tier-4 {
  background-color: var(--tier-mid);
  color: #fff;
}

.tier-5,
.tier-6 {
  background-color: var(--tier-hard);
  color: #fff;
}

.tier-7,
.tier-8 {
  background-color: var(--tier-severe);
  color: #fff;
}

.hero-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 28px;
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.meta-item {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.meta-label {
  font-size: 12px;
  color: var(--text-muted);
}

.meta-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ---------- 三模式最快完成 ---------- */
.section-count {
  font-size: 12px;
  color: var(--text-muted);
}

.wr-summary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 14px;
}

.wr-mode {
  padding: 14px 16px;
}

.wr-mode-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  font-size: 14px;
  font-weight: 700;
}

.mode-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(47, 111, 79, 0.14);
}

.wr-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  font-size: 12px;
  min-width: 0;
}

.wr-item + .wr-item {
  border-top: 1px dashed var(--border-color);
}

.wr-kind {
  flex-shrink: 0;
  padding: 1px 7px;
  border-radius: 4px;
  background-color: var(--bg-subtle);
  font-size: 11px;
  font-weight: 700;
  color: var(--text-secondary);
}

.wr-time {
  flex-shrink: 0;
  font-weight: 700;
  color: var(--accent);
}

.wr-player {
  min-width: 0;
  color: var(--text-secondary);
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.wr-player:hover {
  color: var(--accent);
  text-decoration: underline;
  text-underline-offset: 2px;
}

.wr-none {
  color: var(--text-muted);
}

/* ---------- 模式切换 ---------- */
.mode-tabs {
  display: flex;
  gap: 6px;
  margin: 26px 0 14px;
}

.mode-tab {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 7px 18px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background-color: var(--bg-card);
  font-family: var(--font-base);
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease,
    color 0.15s ease;
}

.mode-tab:hover {
  border-color: var(--accent);
  color: var(--text-primary);
}

.mode-tab.active {
  background-color: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.mode-tab.active .mode-dot {
  background-color: #fff;
  box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.22);
}

/* ---------- PRO / TP 榜：双列并排，紧凑列宽防止横向滑动 ---------- */
.boards-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
  align-items: start;
}

/* 同行面板取消全局「相邻面板上间距」，否则第二列会下沉 */
.boards-grid .panel + .panel {
  margin-top: 0;
}

.time-strong {
  font-weight: 700;
  color: var(--accent);
}

:deep(.upload) {
  font-size: 11px;
  color: var(--text-muted);
}

.board-empty {
  padding: 40px 16px;
  text-align: center;
  font-size: 13px;
  color: var(--text-muted);
}

@media (max-width: 1100px) {
  .boards-grid {
    grid-template-columns: 1fr;
  }

  .boards-grid .panel + .panel {
    margin-top: 16px;
  }
}

@media (max-width: 860px) {
  .map-hero {
    grid-template-columns: 1fr;
  }

  .wr-summary {
    grid-template-columns: 1fr;
  }
}
</style>
