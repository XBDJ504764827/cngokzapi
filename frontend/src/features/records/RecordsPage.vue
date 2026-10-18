<script setup lang="ts">
import { computed } from 'vue'
import { NAlert, NAvatar, NCard, NEmpty, NSpin, NTag } from 'naive-ui'
import { RouterLink } from 'vue-router'
import { useApi } from '@/composables/useApi'
import { MODES } from '@/types/common'
import type { Mode } from '@/types/common'
import { avatarUrl } from '@/utils/avatar'
import { formatDateTimeShort } from '@/utils/format'
import { mapImage } from '@/utils/mapImage'
import { fetchRecords } from './api'
import type { RecordEntry } from './types'

const { data, loading, error } = useApi(fetchRecords)

function rowsFor(mode: Mode): RecordEntry[] {
  return (data.value ?? []).filter((r) => r.mode === mode).slice(0, 20)
}

const hasData = computed(() => (data.value ?? []).length > 0)

/** 地图缩略图加载失败时隐藏，露出底色占位。 */
function onImageError(e: Event): void {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">最新记录</h1>
    <p class="page-sub">三模式并排，各列最新 20 条 · 点击地图查看详情、点击玩家名查看资料</p>
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

        <!-- 条目：左地图缩略图（可点进地图详情），右侧逐行给出全部字段 -->
        <div class="rec-list">
          <div v-for="(r, i) in rowsFor(mode)" :key="i" class="rec-row">
            <RouterLink class="rec-thumb" :to="`/maps/${r.map_id}`" :title="`查看 ${r.map} 地图详情`">
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
              <img :src="mapImage(r.map, 'thumbnails')" :alt="r.map" loading="lazy" @error="onImageError" />
            </RouterLink>

            <div class="rec-body">
              <div class="rec-line">
                <RouterLink class="rec-map" :to="`/maps/${r.map_id}`" :title="r.map">{{ r.map }}</RouterLink>
                <span class="rec-time mono" title="完成耗时">{{ r.time }}</span>
              </div>
              <div class="rec-line">
                <span class="rec-player-wrap">
                  <NAvatar :src="avatarUrl(r.steam_id64)" :size="18" round />
                  <RouterLink
                    class="rec-player"
                    :to="`/profile/${r.steam_id64}`"
                    :title="r.player"
                  >
                    {{ r.player }}
                  </RouterLink>
                </span>
                <span class="rec-meta">
                  <span
                    class="rec-points mono"
                    :class="{ muted: r.points === 0 }"
                    :title="r.points === 0 ? '该成绩未进入计分范围（KZTimer 仅对每张地图/关卡前 1000 名计分）' : '获取分数'"
                  >
                    {{ r.points > 0 ? `${r.points} 分` : '未计分' }}
                  </span>
                  <span class="rec-dot">·</span>
                  <span class="rec-upload mono" :title="`上传于 ${r.created_at}`">
                    {{ formatDateTimeShort(r.created_at) }}
                  </span>
                </span>
              </div>
            </div>
          </div>
        </div>
      </NCard>
    </div>
  </NSpin>
</template>

<style scoped>
.records-columns {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
  align-items: start;
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

.rec-list {
  display: flex;
  flex-direction: column;
}

.rec-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 2px;
}

.rec-row + .rec-row {
  border-top: 1px solid var(--border-color);
}

/* 地图缩略图：点击进入地图详情 */
.rec-thumb {
  position: relative;
  flex-shrink: 0;
  width: 76px;
  height: 44px;
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
  margin-top: 3px;
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

.rec-player-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  flex: 1;
}

.rec-player {
  min-width: 0;
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.rec-player:hover {
  color: var(--accent-hover);
  text-decoration: underline;
  text-underline-offset: 2px;
}

.rec-meta {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  font-size: 12px;
}

.rec-points {
  color: #8a6a2f;
  font-weight: 700;
}

.rec-points.muted {
  color: var(--text-muted);
  font-weight: 500;
}

.rec-dot {
  color: var(--border-strong);
}

.rec-upload {
  color: var(--text-muted);
}

@media (max-width: 1080px) {
  .records-columns {
    grid-template-columns: 1fr;
  }
}
</style>
