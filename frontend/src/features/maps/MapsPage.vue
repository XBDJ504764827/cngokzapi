<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NAlert, NEmpty, NInput, NPagination, NSelect, NSpin, NTag } from 'naive-ui'
import { useRouter } from 'vue-router'
import { useApi } from '@/composables/useApi'
import { useAppStore } from '@/stores/app'
import { mapImage } from '@/utils/mapImage'
import { fetchMaps } from './api'
import type { MapSummary } from './types'

const app = useAppStore()
const router = useRouter()

const query = ref('')
const tier = ref(0)
const page = ref(1)
const PAGE_SIZE = 24

const tierOptions = [
  { label: '所有难度', value: 0 },
  ...Array.from({ length: 8 }, (_, i) => ({ label: `Tier ${i + 1}`, value: i + 1 })),
]

const { data, loading, error, reload } = useApi(() =>
  fetchMaps({ query: query.value.trim(), tier: tier.value, mode: app.currentMode }),
)

watch([query, tier, () => app.currentMode], () => {
  page.value = 1
  void reload()
})

const all = computed(() => data.value ?? [])
const rows = computed(() => all.value.slice((page.value - 1) * PAGE_SIZE, page.value * PAGE_SIZE))
const pageCount = computed(() => Math.max(1, Math.ceil(all.value.length / PAGE_SIZE)))

/** 当前模式的难度（卡片徽章用）。 */
function tierOf(m: MapSummary): number {
  if (app.currentMode === 'SKZ') return m.tier_skz
  if (app.currentMode === 'VNL') return m.tier_vnl
  return m.tier_kzt
}

/** 制作人展示：多个用「, 」连接；上游没给名字时显示"作者未知"。 */
function authorsOf(m: MapSummary): string {
  return m.mappers.length > 0 ? m.mappers.join(', ') : '作者未知'
}

/** 关卡图加载失败时隐藏，露出底色占位（mock 图片阶段容错）。 */
function onImageError(e: Event): void {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

function openMap(id: number): void {
  void router.push(`/maps/${id}`)
}
</script>

<template>
  <div class="page-head page-head-row">
    <div>
      <h1 class="page-title">地图列表</h1>
      <p class="page-sub">当前模式 {{ app.currentMode }} · 点击卡片查看前 100 名</p>
    </div>
    <div class="page-head-actions">
      <NTag type="success" size="medium">共 {{ all.length }} 张</NTag>
    </div>
  </div>

  <div class="toolbar">
    <NInput v-model:value="query" placeholder="搜索地图..." clearable style="max-width: 300px" />
    <NSelect v-model:value="tier" :options="tierOptions" style="width: 160px" />
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>

    <div v-else-if="rows.length" class="maps-grid">
      <div v-for="m in rows" :key="m.id" class="map-card hover-lift" @click="openMap(m.id)">
        <div class="map-thumbnail">
          <!-- 底图：上游缺图时兜底显示地图名，不让卡片留一块灰 -->
          <div class="map-thumb-fallback">
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
            <span>{{ m.name }}</span>
          </div>
          <img :src="mapImage(m.name)" :alt="m.name" loading="lazy" @error="onImageError" />
          <span class="tier-badge" :class="`tier-${tierOf(m)}`">
            {{ tierOf(m) > 0 ? `Tier ${tierOf(m)}` : '未评级' }}
          </span>
        </div>
        <div class="map-content">
          <div class="map-title" :title="m.name">{{ m.name }}</div>
          <div class="map-meta-row">
            <span class="map-author-text" :title="authorsOf(m)">{{ authorsOf(m) }}</span>
            <NTag v-if="m.bonuses > 0" size="small" type="success">Bonus ×{{ m.bonuses }}</NTag>
            <NTag v-else size="small">无 Bonus</NTag>
          </div>
        </div>
      </div>
    </div>

    <NEmpty v-else-if="!loading" description="没有符合条件的地图" style="padding: 48px 0" />
  </NSpin>

  <div v-if="pageCount > 1" class="pager">
    <NPagination v-model:page="page" :page-count="pageCount" :page-size="PAGE_SIZE" :page-sizes="[]" />
    <span class="pager-info">第 {{ page }} / {{ pageCount }} 页</span>
  </div>
</template>

<style scoped>
.maps-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.map-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
  cursor: pointer;
}

.map-thumbnail {
  position: relative;
  width: 100%;
  height: 160px;
  background-color: var(--bg-subtle);
  overflow: hidden;
}

.map-thumb-fallback {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 0 20px;
  color: var(--text-muted);
  font-size: 12px;
}

.map-thumb-fallback span {
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.map-thumbnail img {
  position: relative;
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transition: transform 0.35s ease;
}

.map-card:hover .map-thumbnail img {
  transform: scale(1.04);
}

.tier-badge {
  position: absolute;
  top: 10px;
  left: 10px;
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  background-color: var(--bg-subtle);
  color: var(--text-secondary);
  box-shadow: var(--shadow-sm);
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

.map-content {
  padding: 14px 16px 16px;
}

.map-title {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 10px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.map-meta-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.map-author-text {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pager {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  margin-top: 24px;
}

.pager-info {
  font-size: 13px;
  color: var(--text-muted);
}
</style>
