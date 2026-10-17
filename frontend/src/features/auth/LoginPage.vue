<script setup lang="ts">
import { NAlert, NButton, NCard, useMessage } from 'naive-ui'

const message = useMessage()

// Steam OpenID 接入前（PROJECT §10 下一步）明确告知，不做"假装登录"的模拟动作
function steamLoginUnavailable(): void {
  message.warning('Steam OpenID 登录尚未接入，敬请期待')
}
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">登录</h1>
    <p class="page-sub">使用 Steam 账号登录社区</p>
  </div>

  <NCard title="Steam 登录" style="max-width: 520px">
    登录后可绑定个人资料页、解锁管理功能（删记录、封禁）。
    <NAlert type="info" style="margin: 12px 0">
      Steam OpenID 尚未接入（见 PROJECT §10 下一步）。当前无需登录即可浏览排行榜、地图、
      服务器、记录与封禁名单，个人资料页可直接用 SteamID64 查询。
    </NAlert>
    <NButton type="primary" @click="steamLoginUnavailable">前往 Steam 登录</NButton>
  </NCard>
</template>
