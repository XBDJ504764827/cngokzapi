<script setup lang="ts">
import { computed, h, onUnmounted, watch } from 'vue'
import { NAlert, NDataTable, NSpin, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RouterLink, useRoute } from 'vue-router'
import PlayerCell from '@/components/PlayerCell.vue'
import RankBadge from '@/components/RankBadge.vue'
import { useApi } from '@/composables/useApi'
import { useAppStore } from '@/stores/app'
import { formatFilesize, formatTier } from '@/utils/format'
import { mapImage } from '@/utils/mapImage'
import { fetchMapDetail } from './api'
import type { MapRecordEntry } from './types'

const route = useRoute()
const app = useAppStore()

const mapId = computed(() => Number(route.params.id))
const { data, loading, error, reload } = useApi(() => fetchMapDetail(mapId.value, app.currentMode))

watch([mapId, () => app.currentMode], () => void reload())

// 首次访问会触发整图同步（PROJECT §9.5）：同步完成前自动重试
let timer: ReturnType<typeof setTimeout> | undefined
let retries = 0
watch(data, (d) => {
  if (timer) clearTimeout(timer)
  if (d?.syncing && retries < 6) {
    retries += 1
    timer = setTimeout(() => void reload(), 10000)
  }
})
watch([mapId, () => app.currentMode], () => (retries = 0))
onUnmounted(() => {
  if (timer) clearTimeout(timer)
})

const records = computed(() => data.value?.records ?? [])
const map = computed(() => data.value?.map ?? null)
const authors = computed(() =>
  map.value && map.value.mappers.length > 0 ? map.value.mappers.join(', ') : '作者未知',
)

/** 关卡图加载失败时隐藏，露出底色占位。 */
function onImageError(e: Event): void {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

const columns: DataTableColumns<MapRecordEntry> = [
  {
    title: '排名',
    key: 'rank',
    width: 80,
    render: (row) => h(RankBadge, { rank: row.rank }),
  },
  {
    title: '玩家',
    key: 'player',
    render: (row) => h(PlayerCell, { player: row.player, steamId64: row.steam_id64, avatar: row.avatar }),
  },
  { title: '时间', key: 'time', width: 120, render: (row) => h('span', { class: 'mono' }, row.time) },
  { title: '分数', key: 'points', width: 90, render: (row) => h('span', { class: 'mono' }, row.points) },
  { title: '日期', key: 'date', width: 120 },
]
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">{{ map?.name ?? '地图详情' }}</h1>
    <p class="page-sub">
      <RouterLink to="/maps">← 返回地图列表</RouterLink>
      · 当前模式 {{ app.currentMode }} · 主关前 100 名
    </p>
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>

    <div v-if="map" class="detail-hero panel">
      <div class="detail-thumb">
        <div class="detail-thumb-fallback">
          <svg
            width="22"
            height="22"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M9 3 3 6v15l6-3 6 3 6-3V3l-6 3-6-3Zm0 0v15m6-12v15" />
          </svg>
          <span>{{ map.name }}</span>
        </div>
        <img :src="mapImage(map.name)" :alt="map.name" loading="lazy" @error="onImageError" />
      </div>

      <div class="detail-info">
        <div class="detail-chips">
          <span class="chip">KZT {{ formatTier(map.tier_kzt) }}</span>
          <span class="chip">SKZ {{ formatTier(map.tier_skz) }}</span>
          <span class="chip">VNL {{ formatTier(map.tier_vnl) }}</span>
          <span class="chip">Bonus ×{{ map.bonuses }}</span>
          <span class="chip">{{ formatFilesize(map.filesize) }}</span>
        </div>
        <div class="detail-rows">
          <div class="detail-row">
            <span class="detail-label">制作人</span>
            <span class="detail-value" :title="authors">{{ authors }}</span>
          </div>
          <div v-if="map.workshop_id" class="detail-row">
            <span class="detail-label">创意工坊</span>
            <a
              :href="`https://steamcommunity.com/sharedfiles/filedetails/?id=${map.workshop_id}`"
              target="_blank"
              rel="noreferrer"
              class="accent-link mono"
            >
              {{ map.workshop_id }}
            </a>
          </div>
        </div>
      </div>
    </div>

    <div class="panel">
      <div class="panel-head">
        <div>
          <div class="panel-title">主关排行榜</div>
          <div class="panel-sub">当前模式 {{ app.currentMode }} · 按时间排序</div>
        </div>
        <div class="panel-head-extra">
          <NTag v-if="records.length" type="success" size="small">共 {{ records.length }} 人</NTag>
        </div>
      </div>

      <NAlert v-if="data?.syncing" type="info" style="margin: 16px 20px 0">
        该地图 {{ app.currentMode }} 的完整成绩正在后台同步（首次约十几秒），页面会自动刷新；下方为已同步到的部分成绩。
      </NAlert>

      <NDataTable v-if="records.length" :columns="columns" :data="records" :pagination="{ pageSize: 20 }" />
      <div v-else-if="!loading" class="empty-hint">暂无该地图的成绩数据</div>
    </div>
  </NSpin>
</template>

<style scoped>
.detail-hero {
  display: flex;
  gap: 20px;
  padding: 16px;
  margin-bottom: 16px;
}

.detail-thumb {
  position: relative;
  flex-shrink: 0;
  width: 220px;
  height: 124px;
  border-radius: var(--radius-md);
  background-color: var(--bg-subtle);
  overflow: hidden;
}

.detail-thumb-fallback {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 0 14px;
  font-size: 12px;
  color: var(--text-muted);
}

.detail-thumb-fallback span {
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.detail-thumb img {
  position: relative;
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.detail-info {
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 14px;
}

.detail-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.detail-rows {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-row {
  display: flex;
  gap: 12px;
  font-size: 13px;
}

.detail-label {
  flex-shrink: 0;
  width: 64px;
  color: var(--text-muted);
}

.detail-value {
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-hint {
  padding: 40px 0;
  text-align: center;
  font-size: 14px;
  color: var(--text-muted);
}

@media (max-width: 720px) {
  .detail-hero {
    flex-direction: column;
  }

  .detail-thumb {
    width: 100%;
    height: 160px;
  }
}
</style>
