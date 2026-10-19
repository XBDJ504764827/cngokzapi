<script setup lang="ts">
// 名次徽章：排行榜 / 地图详情共用。前 3 名用金银铜，其余用中性底色。
const props = defineProps<{ rank: number | null }>()

function rankClass(rank: number | null): string {
  if (rank === 1) return 'rank-1'
  if (rank === 2) return 'rank-2'
  if (rank === 3) return 'rank-3'
  return ''
}
</script>

<template>
  <span class="rank" :class="rankClass(props.rank)">#{{ props.rank ?? '—' }}</span>
</template>

<style scoped>
.rank {
  display: inline-block;
  min-width: 42px;
  text-align: center;
  padding: 3px 8px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background-color: var(--bg-subtle);
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 700;
}

.rank-1 {
  background-color: var(--medal-gold);
  color: #4a3200;
}

.rank-2 {
  background-color: var(--medal-silver);
  color: #2c3444;
}

.rank-3 {
  background-color: var(--medal-bronze);
  color: #fff;
}
</style>
