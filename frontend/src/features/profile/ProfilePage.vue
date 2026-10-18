<script setup lang="ts">
import { computed, h, onUnmounted, ref, watch } from 'vue'
import { NAlert, NAvatar, NButton, NCard, NDataTable, NInput, NSpin } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useApi } from '@/composables/useApi'
import { avatarUrl } from '@/utils/avatar'
import { formatNumber } from '@/utils/format'
import { fetchProfile } from './api'
import type { PlayerRecordEntry, PlayerScoreEntry } from './types'

const route = useRoute()
const router = useRouter()

const steamId = computed(() => (route.params.steam_id as string | undefined) ?? '')
const input = ref(steamId.value)

watch(steamId, (v) => (input.value = v))

const { data, loading, error, reload } = useApi(() => fetchProfile(steamId.value), false)

watch(
  steamId,
  (v) => {
    if (v) void reload()
  },
  { immediate: true },
)

// 首次访问会触发后台同步（PROJECT §9.5）：同步中自动重试
let timer: ReturnType<typeof setTimeout> | undefined
let retries = 0
watch(data, (d) => {
  if (timer) clearTimeout(timer)
  if (d?.syncing && retries < 6) {
    retries += 1
    timer = setTimeout(() => void reload(), 8000)
  }
})
watch(steamId, () => (retries = 0))
onUnmounted(() => {
  if (timer) clearTimeout(timer)
})

function submit(): void {
  const id = input.value.trim()
  if (!/^\d{17}$/.test(id)) return
  void router.push(`/profile/${id}`)
}

const scores = computed(() => data.value?.scores ?? [])

const modeColumns: DataTableColumns<PlayerScoreEntry> = [
  { title: '模式', key: 'mode', width: 80 },
  { title: '全球排名', key: 'rank', width: 100, render: (r) => (r.rank === null ? '—' : `#${r.rank}`) },
  { title: '全球积分', key: 'points', width: 120, render: (r) => formatNumber(r.points) },
  {
    title: '本站已同步合计',
    key: 'points_local',
    width: 150,
    render: (r) => (r.points_local === null ? '—' : formatNumber(r.points_local)),
  },
  { title: '完成地图', key: 'maps_completed', width: 100 },
  { title: 'WR（NUB / PRO）', key: 'wrs', width: 140, render: (r) => `${r.wrs_nub} / ${r.wrs_pro}` },
  { title: '已同步成绩', key: 'records', width: 110 },
]

const recordColumns: DataTableColumns<PlayerRecordEntry> = [
  { title: '地图', key: 'map' },
  { title: '模式', key: 'mode', width: 80 },
  { title: '关卡', key: 'stage', width: 70, render: (r) => (r.stage === 0 ? '主关' : `B${r.stage}`) },
  { title: '时间', key: 'time', width: 120, render: (r) => h('span', { class: 'mono' }, r.time) },
  { title: '分数', key: 'points', width: 90 },
  { title: '日期', key: 'date', width: 120 },
]
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">个人资料</h1>
    <p class="page-sub">按 SteamID64 查询，首次查询会在后台从上游同步该玩家数据</p>
  </div>

  <NCard title="玩家查询">
    <div class="toolbar">
      <NInput
        v-model:value="input"
        placeholder="输入 SteamID64（17 位数字）"
        style="max-width: 300px"
        @keyup.enter="submit"
      />
      <NButton type="primary" @click="submit">查询</NButton>
    </div>

    <NAlert v-if="!steamId" type="info">
      直接访问 <code>/profile/&lt;SteamID64&gt;</code>，或点击排行榜 / 封禁名单 / 地图榜中的玩家名进入。
    </NAlert>

    <NSpin v-else :show="loading">
      <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
      <NAlert v-if="data?.syncing" type="info" style="margin-bottom: 12px">
        该玩家数据正在后台同步，页面稍后自动刷新（首次约需十几秒）。
      </NAlert>

      <div v-if="data" class="profile-hero">
        <NAvatar :src="avatarUrl(data.steam_id64, data.avatar)" :size="72" round class="profile-avatar" />
        <div class="profile-info">
          <div class="profile-name">{{ data.name || '未知玩家' }}</div>
          <div class="profile-meta">
            <span class="mono profile-steam">{{ data.steam_id64 }}</span>
            <span v-if="data.region" class="chip">{{ data.region }}</span>
          </div>
        </div>
      </div>

      <div v-if="scores.length" class="panel">
        <div class="panel-head">
          <div>
            <div class="panel-title">模式积分</div>
            <div class="panel-sub">全球积分为上游口径，本站合计仅含已同步成绩</div>
          </div>
        </div>
        <NDataTable :columns="modeColumns" :data="scores" :pagination="false" />
      </div>
      <NAlert v-else-if="data && !data.syncing" type="warning" style="margin-bottom: 12px">
        上游没有该玩家的榜单记录（可能未上榜或 SteamID64 有误）。
      </NAlert>

      <div v-if="data?.recent_records?.length" class="panel">
        <div class="panel-head">
          <div>
            <div class="panel-title">最近成绩</div>
            <div class="panel-sub">来自本站已同步的成绩记录</div>
          </div>
        </div>
        <NDataTable
          :columns="recordColumns"
          :data="data.recent_records"
          :pagination="{ pageSize: 20 }"
        />
      </div>

      <div class="back">
        <RouterLink to="/leaderboard">← 返回排行榜</RouterLink>
      </div>
    </NSpin>
  </NCard>
</template>

<style scoped>
.profile-hero {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 18px 20px;
  margin-bottom: 16px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-sm);
}

.profile-avatar {
  border: 2px solid var(--accent-soft);
  background-color: var(--bg-subtle);
}

.profile-info {
  min-width: 0;
}

.profile-name {
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin-bottom: 6px;
}

.profile-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.profile-steam {
  font-size: 13px;
  color: var(--text-muted);
}

.back {
  margin-top: 16px;
}

.back a {
  color: var(--accent);
  text-decoration: none;
  font-weight: 600;
}
</style>
