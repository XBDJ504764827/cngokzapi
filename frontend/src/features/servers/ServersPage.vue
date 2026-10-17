<script setup lang="ts">
import { computed, ref } from 'vue'
import { NAlert, NButton, NEmpty, NInput, NSpin, NTag, useMessage } from 'naive-ui'
import { useApi } from '@/composables/useApi'
import { mapImage } from '@/utils/mapImage'
import { fetchServers } from './api'
import type { ServerEntry } from './types'

const message = useMessage()
const { data, loading, error } = useApi(fetchServers)

// 视图模式：grid（卡片排列）/ list（列状排列），默认列式（PROJECT §2）
const view = ref<'grid' | 'list'>('list')
const query = ref('')

const rows = computed(() => {
  const q = query.value.trim().toLowerCase()
  return (data.value ?? []).filter((s) => {
    if (!q) return true
    return (
      s.name.toLowerCase().includes(q) ||
      s.map.toLowerCase().includes(q) ||
      (s.group_name ?? '').toLowerCase().includes(q)
    )
  })
})

/** 复制连接地址：页面不渲染明文，仅按钮可用（PROJECT §2 防爬虫约束）。 */
async function copyAddress(ip: string, port: number): Promise<void> {
  try {
    await navigator.clipboard.writeText(`${ip}:${port}`)
    message.success('服务器地址已复制到剪贴板')
  } catch {
    message.error('复制失败，请手动允许剪贴板权限')
  }
}

/** 地图图加载失败时隐藏，露出底色占位。 */
function onImageError(e: Event): void {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

function region(server: ServerEntry): string {
  return [server.country, server.city].filter(Boolean).join(' · ') || '未知地区'
}
</script>

<template>
  <div class="page-head page-head-row">
    <div>
      <h1 class="page-title">服务器列表</h1>
      <p class="page-sub">地址仅经「复制 IP」获取，页面不渲染明文</p>
    </div>
    <div class="view-switcher">
      <button class="view-btn" :class="{ active: view === 'grid' }" title="卡片排列" @click="view = 'grid'">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="7" height="7" />
          <rect x="14" y="3" width="7" height="7" />
          <rect x="3" y="14" width="7" height="7" />
          <rect x="14" y="14" width="7" height="7" />
        </svg>
      </button>
      <button class="view-btn" :class="{ active: view === 'list' }" title="列状排列" @click="view = 'list'">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="8" y1="6" x2="21" y2="6" />
          <line x1="8" y1="12" x2="21" y2="12" />
          <line x1="8" y1="18" x2="21" y2="18" />
          <line x1="3" y1="6" x2="3.01" y2="6" />
          <line x1="3" y1="12" x2="3.01" y2="12" />
          <line x1="3" y1="18" x2="3.01" y2="18" />
        </svg>
      </button>
    </div>
  </div>

  <div class="toolbar">
    <NInput v-model:value="query" placeholder="搜索服务器名 / 地图 / 社区..." clearable style="max-width: 300px" />
    <NTag type="success" size="medium">在线 {{ rows.length }} 台</NTag>
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    <NEmpty v-else-if="!loading && !rows.length" description="暂无在线服务器数据" style="padding: 40px 0" />

    <!-- 卡片排列 -->
    <div v-else-if="view === 'grid'" class="servers-grid">
      <div v-for="s in rows" :key="s.id" class="server-card-grid">
        <div class="server-map-preview">
          <img :src="mapImage(s.map)" :alt="s.map" loading="lazy" @error="onImageError" />
          <div class="server-status-badge"><span class="status-dot" /> 在线</div>
        </div>
        <div class="server-card-content">
          <h3 class="server-name" :title="s.name">{{ s.name }}</h3>
          <div class="server-info-row">
            <span class="server-info-label">当前地图</span>
            <span class="server-info-value" :title="s.map">{{ s.map || '—' }}</span>
          </div>
          <div class="server-info-row">
            <span class="server-info-label">难度 / 区域</span>
            <span class="server-info-value">
              {{ s.map_tier > 0 ? `Tier ${s.map_tier}` : 'Tier —' }} · {{ region(s) }}
            </span>
          </div>
          <div class="server-info-row">
            <span class="server-info-label">玩家</span>
            <span class="server-info-value">{{ s.player_count }}/{{ s.max_players }}</span>
          </div>
          <div class="server-players">
            <div class="server-players-title">在线玩家</div>
            <div class="server-players-list">
              <span v-for="p in s.player_names" :key="p" class="player-chip">{{ p }}</span>
              <span v-if="!s.player_names.length" class="player-chip empty">暂无玩家</span>
            </div>
          </div>
          <div class="server-actions">
            <NButton size="small" @click="copyAddress(s.ip, s.port)">复制 IP</NButton>
          </div>
        </div>
      </div>
    </div>

    <!-- 列状排列 -->
    <div v-else class="servers-list">
      <div v-for="s in rows" :key="s.id" class="server-card-list">
        <div class="server-list-preview">
          <img :src="mapImage(s.map, 'thumbnails')" :alt="s.map" loading="lazy" @error="onImageError" />
          <div class="server-status-badge"><span class="status-dot" /> 在线</div>
        </div>
        <div class="server-list-info">
          <h3 class="server-name" :title="s.name">{{ s.name }}</h3>
          <div class="server-meta">
            <span :title="s.map">{{ s.map || '—' }}</span>
            <span>·</span>
            <span>{{ s.map_tier > 0 ? `Tier ${s.map_tier}` : 'Tier —' }}</span>
            <span>·</span>
            <span>{{ region(s) }}</span>
            <span>·</span>
            <span>{{ s.player_count }}/{{ s.max_players }} 玩家</span>
          </div>
          <div class="server-players-list">
            <span v-for="p in s.player_names" :key="p" class="player-chip">{{ p }}</span>
          </div>
        </div>
        <div class="server-list-actions">
          <NButton size="small" @click="copyAddress(s.ip, s.port)">复制 IP</NButton>
        </div>
      </div>
    </div>
  </NSpin>
</template>

<style scoped>
.page-head-row {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
}

.view-switcher {
  display: flex;
  gap: 4px;
}

.view-btn {
  padding: 8px;
  background-color: transparent;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.view-btn:hover {
  border-color: var(--accent);
  color: var(--text-primary);
}

.view-btn.active {
  background-color: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

/* ---------- 卡片排列 ---------- */
.servers-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 20px;
}

.server-card-grid {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  overflow: hidden;
  transition: all 0.2s ease;
}

.server-card-grid:hover {
  border-color: var(--accent);
  box-shadow: 0 4px 16px rgba(28, 28, 26, 0.06);
}

.server-map-preview {
  position: relative;
  width: 100%;
  height: 180px;
  background-color: var(--bg-subtle);
  overflow: hidden;
}

.server-map-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.server-status-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 12px;
  background-color: rgba(255, 255, 255, 0.95);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  color: #4c8f6b;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: #4c8f6b;
}

.server-card-content {
  padding: 18px;
}

.server-name {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-info-row {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 10px;
  font-size: 14px;
}

.server-info-label {
  color: var(--text-muted);
  white-space: nowrap;
}

.server-info-value {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-players {
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.server-players-title {
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.server-players-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.player-chip {
  padding: 2px 8px;
  background-color: var(--bg-subtle);
  border-radius: 10px;
  font-size: 12px;
  color: var(--text-secondary);
}

.player-chip.empty {
  color: var(--text-muted);
}

.server-actions {
  display: flex;
  gap: 8px;
  margin-top: 14px;
}

/* ---------- 列状排列 ---------- */
.servers-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.server-card-list {
  display: flex;
  align-items: center;
  gap: 16px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 12px;
  transition: all 0.2s ease;
}

.server-card-list:hover {
  border-color: var(--accent);
}

.server-list-preview {
  position: relative;
  flex-shrink: 0;
  width: 120px;
  height: 68px;
  border-radius: 6px;
  background-color: var(--bg-subtle);
  overflow: hidden;
}

.server-list-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.server-list-preview .server-status-badge {
  top: 6px;
  right: 6px;
  padding: 1px 8px;
  font-size: 11px;
}

.server-list-info {
  min-width: 0;
  flex: 1;
}

.server-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.server-list-actions {
  flex-shrink: 0;
}

@media (max-width: 600px) {
  .server-card-list {
    flex-direction: column;
    align-items: stretch;
  }

  .server-list-preview {
    width: 100%;
    height: 140px;
  }
}
</style>
