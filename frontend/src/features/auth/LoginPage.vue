<script setup lang="ts">
import { NButton, NCard, useMessage } from 'naive-ui'

const message = useMessage()

function mockLogin(): void {
  message.info('将跳转 Steam OpenID（对接中），当前为模拟')
}
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">登录</h1>
    <p class="page-sub">使用 Steam 账号登录社区</p>
  </div>

  <NCard title="Steam 登录" style="max-width: 480px">
    登录后同步头像、同步游戏时长，并解锁管理功能（删记录、封禁）。
    <div style="margin-top: 12px">
      <NButton type="primary" @click="mockLogin">前往 Steam 登录</NButton>
    </div>
  </NCard>
</template>
