<script setup lang="ts">
import { NCard, NGrid, NGi, NStatistic } from 'naive-ui'
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">中国 GOKZ 社区</h1>
    <p class="page-sub">记录你的每一次跳跃，见证技术的极限</p>
  </div>

  <NGrid :cols="4" :x-gap="12" responsive="screen">
    <NGi><NCard><NStatistic label="活跃玩家" value="1,234" /></NCard></NGi>
    <NGi><NCard><NStatistic label="地图总数" value="567" /></NCard></NGi>
    <NGi><NCard><NStatistic label="在线服务器" value="12" /></NCard></NGi>
    <NGi><NCard><NStatistic label="总记录数" value="89,234" /></NCard></NGi>
  </NGrid>

  <NCard title="关于 CNGOKZ" style="margin-top: 16px">
    社区统计、服务器状态与公告接口（modules/home）待后端接入后替换此处模拟数字。
  </NCard>
</template>
