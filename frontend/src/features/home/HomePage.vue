<script setup lang="ts">
import { computed } from 'vue'
import { NAlert, NCard, NGrid, NGi, NStatistic } from 'naive-ui'
import { useApi } from '@/composables/useApi'
import { formatNumber } from '@/utils/format'
import { fetchStats } from './api'
import { announcements } from './announcements'

const { data, error } = useApi(fetchStats)

// 布局与重构前保持一致：4 个统计卡 + 「关于 CNGOKZ」卡 + 「最新公告」列表
const cards = computed(() => [
  { label: '活跃玩家', value: num(data.value?.players) },
  { label: '地图总数', value: num(data.value?.maps) },
  { label: '在线服务器', value: num(data.value?.servers_online) },
  { label: '总记录数', value: num(data.value?.records) },
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
  <div class="page-head">
    <h1 class="page-title">中国 GOKZ 社区</h1>
    <p class="page-sub">记录你的每一次跳跃，见证技术的极限</p>
  </div>

  <NGrid :cols="4" :x-gap="12" responsive="screen">
    <NGi v-for="c in cards" :key="c.label">
      <NCard><NStatistic :label="c.label" :value="c.value" /></NCard>
    </NGi>
  </NGrid>

  <NCard title="关于 CNGOKZ" style="margin-top: 16px">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    社区统计、在线服务器与最新记录均来自本站已同步的真实数据（数据源：GOKZ.TOP / KZTimer Global API），
    统计时间 {{ updatedAt }}。榜单、地图、服务器、最新记录与封禁名单见顶部导航；
    玩家资料页可直接访问 <code>/profile/&lt;SteamID64&gt;</code>，或点击任意列表中的玩家名进入。
  </NCard>

  <section class="section">
    <div class="section-header">
      <h2 class="section-title">最新公告</h2>
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
.section {
  margin-top: 32px;
}

.section-header {
  margin-bottom: 14px;
}

.section-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
}

.announcement-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.announcement-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 20px;
  transition: all 0.2s ease;
}

.announcement-card:hover {
  border-color: var(--accent);
}

.announcement-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
}

.announcement-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.announcement-date {
  font-size: 13px;
  color: var(--text-muted);
  white-space: nowrap;
}

.announcement-content {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
}
</style>
