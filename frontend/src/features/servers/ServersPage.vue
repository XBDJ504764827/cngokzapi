<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NAlert, NButton, NEmpty, NInput, NSpin, NTag, useMessage } from 'naive-ui'
import { useApi } from '@/composables/useApi'
import { mapImage } from '@/utils/mapImage'
import { fetchServers } from './api'
import type { ServerEntry } from './types'

const message = useMessage()
const { data, loading, error } = useApi(fetchServers)

// 视图模式：grid（卡片排列）/ list（列状排列），默认卡片排列
const view = ref<'grid' | 'list'>('grid')
const query = ref('')
const activeCategory = ref('')

// 按社区分类（后端 category）：计数倒序，同数量按名称排序
const categories = computed(() => {
  const counts = new Map<string, number>()
  for (const s of data.value ?? []) counts.set(s.category, (counts.get(s.category) ?? 0) + 1)
  return [...counts.entries()]
    .map(([name, count]) => ({ name, count }))
    .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name, 'zh-CN'))
})

// 上游下线某社区后自动回到「全部」，避免停在空筛选
watch(categories, (list) => {
  if (activeCategory.value && !list.some((c) => c.name === activeCategory.value)) {
    activeCategory.value = ''
  }
})

const rows = computed(() => {
  const q = query.value.trim().toLowerCase()
  const cat = activeCategory.value
  return (data.value ?? []).filter((s) => {
    if (cat && s.category !== cat) return false
    if (!q) return true
    return (
      s.name.toLowerCase().includes(q) ||
      s.map.toLowerCase().includes(q) ||
      s.category.toLowerCase().includes(q) ||
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

// CS 客户端 AppID（用户指定 2026-10 更新为 4465480）。
const CS_APP_ID = '4465480'

/** 拉起 Steam 客户端并连接服务器；地址仅在点击时拼接，不写入 DOM（PROJECT §2）。 */
function joinServer(ip: string, port: number): void {
  window.location.href = `steam://rungameid/${CS_APP_ID}//+connect%20${ip}:${port}`
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
    <div class="page-head-actions">
      <NTag type="success" size="medium">在线 {{ rows.length }} 台</NTag>
      <div class="view-switcher">
        <button class="view-btn" :class="{ active: view === 'grid' }" title="卡片排列" @click="view = 'grid'">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="7" />
            <rect x="14" y="3" width="7" height="7" />
            <rect x="3" y="14" width="7" height="7" />
            <rect x="14" y="14" width="7" height="7" />
          </svg>
        </button>
        <button class="view-btn" :class="{ active: view === 'list' }" title="列状排列" @click="view = 'list'">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
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
  </div>

  <div class="category-bar">
    <button class="cat-chip" :class="{ active: activeCategory === '' }" @click="activeCategory = ''">
      全部<span class="cat-count">{{ data?.length ?? 0 }}</span>
    </button>
    <button
      v-for="c in categories"
      :key="c.name"
      class="cat-chip"
      :class="{ active: activeCategory === c.name }"
      :title="c.name"
      @click="activeCategory = c.name"
    >
      <span class="cat-name">{{ c.name }}</span>
      <span class="cat-count">{{ c.count }}</span>
    </button>
  </div>

  <div class="toolbar">
    <NInput v-model:value="query" placeholder="搜索服务器名 / 地图 / 社区..." clearable style="max-width: 320px" />
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    <NEmpty v-else-if="!loading && !rows.length" description="暂无在线服务器数据" style="padding: 48px 0" />

    <!-- 卡片排列 -->
    <div v-else-if="view === 'grid'" class="servers-grid">
      <div v-for="s in rows" :key="s.id" class="server-card-grid hover-lift">
        <div class="server-map-preview">
          <div class="thumb-fallback">
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M9 3 3 6v15l6-3 6 3 6-3V3l-6 3-6-3Zm0 0v15m6-12v15" />
            </svg>
            <span>{{ s.map || '未知地图' }}</span>
          </div>
          <img :src="mapImage(s.map)" :alt="s.map" loading="lazy" @error="onImageError" />
          <span class="cat-tag">{{ s.category }}</span>
          <div class="server-status-badge"><span class="status-dot" /> 在线</div>
        </div>
        <div class="server-card-content">
          <div class="server-card-head">
            <h3 class="server-name" :title="s.name">{{ s.name }}</h3>
            <span class="server-count mono">{{ s.player_count }}/{{ s.max_players }}</span>
          </div>
          <div class="server-info-row">
            <span class="server-info-label">当前地图</span>
            <span class="server-info-value" :title="s.map">{{ s.map || '—' }}</span>
          </div>
          <div class="server-info-row">
            <span class="server-info-label">数据来源</span>
            <span class="server-info-value">{{ s.source === 'plugin' ? '插件上报' : 'Steam 公开 API' }}</span>
          </div>
          <div class="server-info-row">
            <span class="server-info-label">难度 / 区域</span>
            <span class="server-info-value">
              {{ s.map_tier > 0 ? `Tier ${s.map_tier}` : 'Tier —' }} · {{ region(s) }}
            </span>
          </div>
          <div class="server-players">
            <div class="server-players-title">在线玩家</div>
            <div class="server-players-list">
              <span v-for="p in s.player_names" :key="p" class="player-chip">{{ p }}</span>
              <span v-if="!s.player_names.length" class="player-chip empty">暂无玩家</span>
            </div>
          </div>
          <div class="server-actions">
            <NButton size="small" type="primary" @click="joinServer(s.ip, s.port)">加入游戏</NButton>
            <NButton size="small" @click="copyAddress(s.ip, s.port)">复制 IP</NButton>
          </div>
        </div>
      </div>
    </div>

    <!-- 列状排列 -->
    <div v-else class="servers-list">
      <div v-for="s in rows" :key="s.id" class="server-card-list">
        <div class="server-list-preview">
          <div class="thumb-fallback small">
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
          <img :src="mapImage(s.map, 'thumbnails')" :alt="s.map" loading="lazy" @error="onImageError" />
          <div class="server-status-badge"><span class="status-dot" /> 在线</div>
        </div>
        <div class="server-list-info">
          <h3 class="server-name" :title="s.name">{{ s.name }}</h3>
          <div class="server-meta">
            <span class="cat-tag small">{{ s.category }}</span>
            <span class="src-tag" :class="s.source">{{ s.source === 'plugin' ? '插件' : 'Steam' }}</span>
            <span class="meta-strong" :title="s.map">{{ s.map || '—' }}</span>
            <span class="meta-sep">·</span>
            <span>{{ s.map_tier > 0 ? `Tier ${s.map_tier}` : 'Tier —' }}</span>
            <span class="meta-sep">·</span>
            <span>{{ region(s) }}</span>
          </div>
          <div class="server-players-list">
            <span v-for="p in s.player_names" :key="p" class="player-chip">{{ p }}</span>
            <span v-if="!s.player_names.length" class="player-chip empty">暂无玩家</span>
          </div>
        </div>
        <div class="server-list-actions">
          <span class="server-count mono">{{ s.player_count }}/{{ s.max_players }}</span>
          <div class="list-buttons">
            <NButton size="small" type="primary" @click="joinServer(s.ip, s.port)">加入游戏</NButton>
            <NButton size="small" @click="copyAddress(s.ip, s.port)">复制 IP</NButton>
          </div>
        </div>
      </div>
    </div>
  </NSpin>
</template>

<style scoped>
/* 社区分类筛选：横向滚动，计数倒序 */
.category-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 14px;
  overflow-x: auto;
  padding-bottom: 2px;
  scrollbar-width: none;
}

.category-bar::-webkit-scrollbar {
  display: none;
}

.cat-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  max-width: 200px;
  padding: 5px 12px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 999px;
  font-family: var(--font-base);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease,
    color 0.15s ease;
}

.cat-chip:hover {
  border-color: var(--accent);
  color: var(--text-primary);
}

.cat-chip.active {
  background-color: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent);
  font-weight: 600;
}

.cat-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cat-count {
  flex-shrink: 0;
  padding: 0 7px;
  border-radius: 999px;
  background-color: var(--bg-subtle);
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.cat-chip.active .cat-count {
  background-color: rgba(47, 111, 79, 0.14);
  color: var(--accent);
}

/* 卡片 / 列表上的社区标签 */
.cat-tag {
  position: absolute;
  top: 12px;
  left: 12px;
  max-width: calc(100% - 110px);
  padding: 3px 10px;
  border-radius: 999px;
  background-color: rgba(255, 255, 255, 0.95);
  box-shadow: var(--shadow-sm);
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cat-tag.small {
  position: static;
  flex-shrink: 0;
  max-width: 160px;
  padding: 2px 9px;
  background-color: var(--accent-soft);
  box-shadow: none;
  font-size: 12px;
  color: var(--accent);
}

/* 数据来源标签：插件=绿，Steam=中性 */
.src-tag {
  flex-shrink: 0;
  padding: 2px 8px;
  border-radius: 999px;
  background-color: var(--bg-subtle);
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.src-tag.plugin {
  background-color: var(--accent-soft);
  color: var(--accent);
}

.view-switcher {
  display: flex;
  gap: 4px;
}

.view-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 7px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease,
    color 0.15s ease;
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

/* 缩略图兜底：上游缺图时显示地图名 / 图标，不留灰块 */
.thumb-fallback {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 0 14px;
  color: var(--text-muted);
  font-size: 12px;
}

.thumb-fallback span {
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.thumb-fallback.small {
  flex-direction: row;
}

.thumb-fallback.small span {
  display: none;
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
  border-radius: var(--radius-md);
  overflow: hidden;
}

.server-map-preview {
  position: relative;
  width: 100%;
  height: 170px;
  background-color: var(--bg-subtle);
  overflow: hidden;
}

.server-map-preview img {
  position: relative;
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transition: transform 0.35s ease;
}

.server-card-grid:hover .server-map-preview img {
  transform: scale(1.04);
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
  border-radius: 999px;
  box-shadow: var(--shadow-sm);
  font-size: 12px;
  font-weight: 600;
  color: var(--tier-easy);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: var(--tier-easy);
  box-shadow: 0 0 0 3px rgba(76, 143, 107, 0.16);
}

.server-card-content {
  padding: 16px 18px 18px;
}

.server-card-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
}

.server-name {
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-count {
  flex-shrink: 0;
  font-size: 13px;
  font-weight: 700;
  color: var(--accent);
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
  border-radius: 999px;
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
  /* 主操作靠左、次操作（复制 IP）靠右，避免两个按钮挤在一起 */
  justify-content: space-between;
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
  border-radius: var(--radius-md);
  padding: 12px;
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease;
}

.server-card-list:hover {
  border-color: var(--accent);
  box-shadow: var(--shadow-sm);
}

.server-list-preview {
  position: relative;
  flex-shrink: 0;
  width: 120px;
  height: 68px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-subtle);
  overflow: hidden;
}

.server-list-preview img {
  position: relative;
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
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.meta-strong {
  max-width: 320px;
  color: var(--text-primary);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.meta-sep {
  color: var(--border-strong);
}

.server-list-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  flex-shrink: 0;
}

.list-buttons {
  display: flex;
  gap: 8px;
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

  .server-list-actions {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }

  .list-buttons {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
