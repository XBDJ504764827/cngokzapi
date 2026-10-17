<script setup lang="ts">
import { computed, h, onUnmounted, watch } from 'vue'
import { NAlert, NCard, NDataTable, NSpin, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RouterLink, useRoute } from 'vue-router'
import PlayerCell from '@/components/PlayerCell.vue'
import { useApi } from '@/composables/useApi'
import { useAppStore } from '@/stores/app'
import { formatFilesize, formatTier } from '@/utils/format'
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

const columns: DataTableColumns<MapRecordEntry> = [
  { title: '排名', key: 'rank', width: 80 },
  {
    title: '玩家',
    key: 'player',
    render: (row) => h(PlayerCell, { player: row.player, steamId64: row.steam_id64, avatar: row.avatar }),
  },
  { title: '时间', key: 'time', width: 120 },
  { title: '分数', key: 'points', width: 90 },
  { title: '日期', key: 'date', width: 120 },
]
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">{{ map?.name ?? '地图详情' }}</h1>
    <p class="page-sub">
      <RouterLink to="/maps" class="back-link">← 返回地图列表</RouterLink>
      · 当前模式 {{ app.currentMode }} · 主关前 100 名
    </p>
  </div>

  <NSpin :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>

    <NCard v-if="map" title="基本信息" style="margin-bottom: 16px">
      制作人：{{ map.mappers.length ? map.mappers.join(', ') : '作者未知' }} ·
      KZT {{ formatTier(map.tier_kzt) }} · SKZ {{ formatTier(map.tier_skz) }} ·
      VNL {{ formatTier(map.tier_vnl) }} · Bonus {{ map.bonuses }} ·
      大小 {{ formatFilesize(map.filesize) }}
      <template v-if="map.workshop_id">
        ·
        <a
          :href="`https://steamcommunity.com/sharedfiles/filedetails/?id=${map.workshop_id}`"
          target="_blank"
          rel="noreferrer"
          class="workshop-link"
        >
          创意工坊 {{ map.workshop_id }}
        </a>
      </template>
    </NCard>

    <NCard title="主关排行榜">
      <template #header-extra>
        <NTag v-if="records.length" type="success" size="small">共 {{ records.length }} 人</NTag>
      </template>
      <NAlert v-if="data?.syncing" type="info" style="margin-bottom: 12px">
        该地图 {{ app.currentMode }} 的完整成绩正在后台同步（首次约十几秒），页面会自动刷新；下方为已同步到的部分成绩。
      </NAlert>
      <NDataTable :columns="columns" :data="records" :pagination="{ pageSize: 20 }" />
    </NCard>
  </NSpin>
</template>

<style scoped>
.back-link,
.workshop-link {
  color: var(--accent);
  text-decoration: none;
  font-weight: 600;
}
</style>
