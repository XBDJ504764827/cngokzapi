<script setup lang="ts">
import { computed } from 'vue'
import { NAlert, NAvatar, NCard, NEmpty, NSpin, NTag } from 'naive-ui'
import { RouterLink } from 'vue-router'
import { useApi } from '@/composables/useApi'
import { MODES } from '@/types/common'
import type { Mode } from '@/types/common'
import { avatarUrl } from '@/utils/avatar'
import { fetchRecords } from './api'
import type { RecordEntry } from './types'

const { data, loading, error } = useApi(fetchRecords)

function rowsFor(mode: Mode): RecordEntry[] {
  return (data.value ?? []).filter((r) => r.mode === mode).slice(0, 20)
}

const hasData = computed(() => (data.value ?? []).length > 0)
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">最新记录</h1>
    <p class="page-sub">三模式并排，各列最新 20 条 · 点击玩家名查看资料</p>
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

        <!-- 两行式条目：一列内即可完整展示玩家/地图/时间/分数/日期，无需横向滚动 -->
        <div class="rec-list">
          <div v-for="(r, i) in rowsFor(mode)" :key="i" class="rec-row">
            <NAvatar :src="avatarUrl(r.steam_id64)" :size="26" round class="rec-avatar" />
            <div class="rec-body">
              <div class="rec-line">
                <RouterLink class="rec-player" :to="`/profile/${r.steam_id64}`" :title="r.player">
                  {{ r.player }}
                </RouterLink>
                <span class="rec-time mono">{{ r.time }}</span>
              </div>
              <div class="rec-line">
                <span class="rec-map" :title="r.map">{{ r.map }}</span>
                <span class="rec-meta">
                  <span class="rec-points mono">{{ r.points }} 分</span>
                  <span class="rec-dot">·</span>
                  <span class="rec-date mono">{{ r.date }}</span>
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

.rec-avatar {
  flex-shrink: 0;
  background-color: var(--bg-subtle);
}

.rec-body {
  flex: 1;
  min-width: 0;
}

.rec-line {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-width: 0;
}

.rec-line + .rec-line {
  margin-top: 2px;
}

.rec-player {
  min-width: 0;
  font-size: 13px;
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

.rec-time {
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 700;
  color: var(--accent);
}

.rec-map {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.rec-meta {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-shrink: 0;
  font-size: 12px;
  color: var(--text-muted);
}

.rec-points {
  color: #8a6a2f;
  font-weight: 600;
}

.rec-dot {
  color: var(--border-strong);
}

@media (max-width: 1080px) {
  .records-columns {
    grid-template-columns: 1fr;
  }
}
</style>
