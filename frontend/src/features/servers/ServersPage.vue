<script setup lang="ts">
import { NAlert, NCard } from 'naive-ui'
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">服务器列表</h1>
    <p class="page-sub">默认列式展示，地址仅经复制 IP 获取，页面不渲染明文地址</p>
  </div>

  <NCard title="待接入">
    服务器实时接口（modules/servers）待后端接入。
    <NAlert type="warning" style="margin-top: 12px">
      安全约束：卡片不得直接展示服务器地址，仅保留复制 IP 入口（防爬虫），见 PROJECT.md。
    </NAlert>
  </NCard>
</template>
