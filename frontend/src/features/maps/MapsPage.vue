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
  <div class="page-head">
    <h1 class="page-title">地图列表</h1>
    <p class="page-sub">当前模式 {{ app.currentMode }} · 共 {{ all.length }} 张 · 点击卡片查看前 100 名</p>
  </div>

  <div class="toolbar">
    <NInput v-model:value="query" placeholder="搜索地图..." clearable style="max-width: 300px" />
    <NSelect v-model:value="tier" :options="tierOptions" style="width: 160px" />
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>

    <div v-else-if="rows.length" class="maps-grid">
      <div v-for="m in rows" :key="m.id" class="map-card" @click="openMap(m.id)">
        <div class="map-thumbnail">
          <img :src="mapImage(m.name)" :alt="m.name" loading="lazy" @error="onImageError" />
        </div>
        <div class="map-content">
          <div class="map-title" :title="m.name">{{ m.name }}</div>
          <div class="map-meta-row">
            <span class="tier-badge" :class="`tier-${tierOf(m)}`">
              {{ tierOf(m) > 0 ? `Tier ${tierOf(m)}` : '未评级' }}
            </span>
            <span class="map-author-text" :title="authorsOf(m)">{{ authorsOf(m) }}</span>
          </div>
          <div class="map-flags">
            <NTag v-if="m.bonuses > 0" size="small" type="success">Bonus ×{{ m.bonuses }}</NTag>
            <NTag v-else size="small">无 Bonus</NTag>
          </div>
        </div>
      </div>
    </div>

    <NEmpty v-else-if="!loading" description="没有符合条件的地图" style="padding: 40px 0" />
  </NSpin>

  <div v-if="pageCount > 1" class="pager">
    <NPagination v-model:page="page" :page-count="pageCount" :page-size="PAGE_SIZE" :page-sizes="[]" />
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
  border-radius: 10px;
  overflow: hidden;
  transition: all 0.2s ease;
  cursor: pointer;
}

.map-card:hover {
  border-color: var(--accent);
  box-shadow: 0 4px 16px rgba(28, 28, 26, 0.06);
}

.map-thumbnail {
  width: 100%;
  height: 160px;
  background-color: var(--bg-subtle);
  overflow: hidden;
}

.map-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.map-content {
  padding: 18px;
}

.map-title {
  font-size: 16px;
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
  margin-bottom: 10px;
}

.tier-badge {
  padding: 3px 10px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  background-color: var(--bg-subtle);
  color: var(--text-secondary);
}

.tier-1,
.tier-2 {
  background-color: #4c8f6b;
  color: #fff;
}

.tier-3,
.tier-4 {
  background-color: #b8862f;
  color: #fff;
}

.tier-5,
.tier-6 {
  background-color: #b5533c;
  color: #fff;
}

.tier-7,
.tier-8 {
  background-color: #8e2a2a;
  color: #fff;
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
  justify-content: center;
  margin-top: 20px;
}
</style>
