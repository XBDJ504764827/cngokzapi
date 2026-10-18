<script setup lang="ts">
import { computed } from 'vue'
import { NAlert, NCard, NGrid, NGi, NStatistic } from 'naive-ui'
import { RouterLink } from 'vue-router'
import { useApi } from '@/composables/useApi'
import { formatNumber } from '@/utils/format'
import { fetchStats } from './api'
import { announcements } from './announcements'

const { data, error } = useApi(fetchStats)

// 每张统计卡配一个线性图标（细描边，克制不抢视线）
const ICON_PLAYERS =
  'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 3a4 4 0 1 1 0 8 4 4 0 0 1 0-8Zm13 18v-2a4 4 0 0 0-3-3.87'
const ICON_MAPS = 'M9 3 3 6v15l6-3 6 3 6-3V3l-6 3-6-3Zm0 0v15m6-12v15'
const ICON_SERVERS = 'M4 4h16v6H4zM4 14h16v6H4zM8 7h.01M8 17h.01'
const ICON_RECORDS = 'M12 8v4l3 3m6-3a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z'

// 布局不变：4 个统计卡 + 「关于 CNGOKZ」卡 + 「最新公告」列表
const cards = computed(() => [
  { label: '活跃玩家', value: num(data.value?.players), icon: ICON_PLAYERS },
  { label: '地图总数', value: num(data.value?.maps), icon: ICON_MAPS },
  { label: '在线服务器', value: num(data.value?.servers_online), icon: ICON_SERVERS },
  { label: '总记录数', value: num(data.value?.records), icon: ICON_RECORDS },
])

function num(n: number | undefined): string {
  return n === undefined ? '—' : formatNumber(n)
}

const updatedAt = computed(() => {
  const t = data.value?.updated_at
  return t ? new Date(t).toLocaleString('zh-CN') : '—'
})
</script>

<template>
  <div class="page-head hero">
    <span class="page-eyebrow">CNGOKZ · 中国 GOKZ 社区</span>
    <h1 class="page-title">中国 GOKZ 社区</h1>
    <p class="page-sub">记录你的每一次跳跃，见证技术的极限</p>
    <div class="hero-actions">
      <RouterLink to="/leaderboard" class="hero-btn hero-btn-primary">查看排行榜</RouterLink>
      <RouterLink to="/maps" class="hero-btn">浏览地图</RouterLink>
    </div>
  </div>

  <NGrid cols="1 s:2 m:4" :x-gap="14" responsive="screen">
    <NGi v-for="c in cards" :key="c.label">
      <NCard class="stat-card hover-lift">
        <span class="stat-icon">
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path :d="c.icon" />
          </svg>
        </span>
        <NStatistic :label="c.label" :value="c.value" />
      </NCard>
    </NGi>
  </NGrid>

  <NCard title="关于 CNGOKZ" class="section-card">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    <div class="intro-grid">
      <div>
        <p class="intro-text">
          CNGOKZ 是面向中文 KZ 玩家的社区平台，收录 GOKZ 生态内的服务器成绩，提供地图排行榜、
          玩家资料查询与跨服务器成绩统计。玩家在服内完成地图后，成绩会经上游全球 API 同步到本站并参与排名。
        </p>
        <p class="intro-text">
          平台支持 KZT / SKZ / VNL 三种模式独立计分，每种模式拥有各自的排行榜与积分体系，互不影响。
          管理员可对记录进行审核删除、对作弊玩家进行封禁处理 —— 被封禁玩家的历史成绩不会从记录中移除，
          但不再计入其他玩家的排名比较。
        </p>
      </div>
      <ul class="intro-facts">
        <li class="fact-item">
          <span class="fact-label">数据来源</span>
          <span class="fact-value">GOKZ.TOP / KZTimer Global API</span>
        </li>
        <li class="fact-item">
          <span class="fact-label">覆盖模式</span>
          <span class="fact-value">KZT / SKZ / VNL 独立计分</span>
        </li>
        <li class="fact-item">
          <span class="fact-label">收录内容</span>
          <span class="fact-value">排行榜 · 地图 · 服务器 · 记录 · 封禁</span>
        </li>
        <li class="fact-item">
          <span class="fact-label">统计时间</span>
          <span class="fact-value mono">{{ updatedAt }}</span>
        </li>
      </ul>
    </div>
  </NCard>

  <section class="section section-announcements">
    <div class="section-header">
      <h2 class="section-title">最新公告</h2>
      <span class="section-count">共 {{ announcements.length }} 条</span>
    </div>
    <div class="announcement-list">
      <div v-for="a in announcements" :key="a.id" class="announcement-card">
        <div class="announcement-header">
          <h3 class="announcement-title">{{ a.title }}</h3>
          <span class="announcement-date">{{ a.date }}</span>
        </div>
        <p class="announcement-content">{{ a.content }}</p>
      </div>
    </div>
  </section>
</template>

<style scoped>
/* Hero：用极浅的墨绿晕染建立首屏层次，仍然保持白底简约 */
.hero {
  position: relative;
  padding: 8px 0 4px;
}

.hero::before {
  content: '';
  position: absolute;
  inset: -60px -80px auto -80px;
  height: 220px;
  background: radial-gradient(520px 200px at 18% 40%, var(--accent-wash), transparent 70%);
  pointer-events: none;
  z-index: -1;
}

.hero-actions {
  display: flex;
  gap: 10px;
  margin-top: 20px;
  flex-wrap: wrap;
}

.hero-btn {
  display: inline-flex;
  align-items: center;
  padding: 9px 18px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-md);
  background-color: var(--bg-card);
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  text-decoration: none;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease,
    color 0.15s ease;
}

.hero-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.hero-btn-primary {
  border-color: var(--accent);
  background-color: var(--accent);
  color: #fff;
}

.hero-btn-primary:hover {
  background-color: var(--accent-hover);
  color: #fff;
}

.stat-card {
  height: 100%;
}

.stat-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  margin-bottom: 14px;
  border-radius: var(--radius-sm);
  background-color: var(--accent-soft);
  color: var(--accent);
}

.section-card {
  margin-top: 20px;
  margin-bottom: 40px;
}

.intro-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.6fr) minmax(0, 1fr);
  gap: 28px;
}

.intro-text {
  font-size: 15px;
  line-height: 1.8;
  color: var(--text-secondary);
  margin-bottom: 14px;
}

.intro-text:last-child {
  margin-bottom: 0;
}

.intro-facts {
  list-style: none;
  padding: 4px 0 0 24px;
  border-left: 1px solid var(--border-color);
}

.fact-item {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 9px 0;
}

.fact-item + .fact-item {
  border-top: 1px dashed var(--border-color);
}

.fact-label {
  font-size: 12px;
  color: var(--text-muted);
}

.fact-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.section-announcements {
  margin-bottom: 8px;
}

.section-count {
  font-size: 12px;
  color: var(--text-muted);
}

.announcement-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.announcement-card {
  position: relative;
  padding: 18px 20px 18px 22px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease;
}

/* 左侧墨绿细条：hover 时出现，作为轻提示 */
.announcement-card::before {
  content: '';
  position: absolute;
  left: 0;
  top: 14px;
  bottom: 14px;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background-color: var(--accent);
  opacity: 0;
  transition: opacity 0.18s ease;
}

.announcement-card:hover {
  border-color: var(--accent);
  box-shadow: var(--shadow-sm);
}

.announcement-card:hover::before {
  opacity: 1;
}

.announcement-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 8px;
}

.announcement-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.announcement-date {
  flex-shrink: 0;
  padding: 2px 10px;
  border-radius: 999px;
  background-color: var(--bg-subtle);
  font-size: 12px;
  color: var(--text-muted);
}

.announcement-content {
  font-size: 14px;
  line-height: 1.7;
  color: var(--text-secondary);
}

@media (max-width: 860px) {
  .intro-grid {
    grid-template-columns: 1fr;
  }

  .intro-facts {
    padding: 4px 0 0;
    border-left: none;
    border-top: 1px solid var(--border-color);
    margin-top: 6px;
  }
}
</style>
