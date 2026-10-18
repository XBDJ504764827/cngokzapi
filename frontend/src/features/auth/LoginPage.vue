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

  <NCard title="Steam 登录" class="login-card">
    <div class="login-hero">
      <span class="login-mark">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <path
            d="M12 2a10 10 0 0 1 10 10 10 10 0 0 1-10 10A10 10 0 0 1 2 12 10 10 0 0 1 12 2m0 18c4.41 0 8-3.59 8-8s-3.59-8-8-8-8 3.59-8 8 3.59 8 8 8m-2-13.5a2.5 2.5 0 0 1 2.5 2.5 2.5 2.5 0 0 1-2.5 2.5A2.5 2.5 0 0 1 7.5 9a2.5 2.5 0 0 1 2.5-2.5m5 5a2.5 2.5 0 0 1 2.5 2.5 2.5 2.5 0 0 1-2.5 2.5A2.5 2.5 0 0 1 12.5 14a2.5 2.5 0 0 1 2.5-2.5z"
          />
        </svg>
      </span>
      <div>
        <div class="login-title">登录后可绑定个人资料页、解锁管理功能（删记录、封禁）。</div>
        <div class="login-sub">当前无需登录即可浏览排行榜、地图、服务器、记录与封禁名单。</div>
      </div>
    </div>

    <NAlert type="info" style="margin: 14px 0">
      Steam OpenID 尚未接入（见 PROJECT §10 下一步）。个人资料页可直接用 SteamID64 查询。
    </NAlert>
    <NButton type="primary" @click="steamLoginUnavailable">前往 Steam 登录</NButton>
  </NCard>
</template>

<style scoped>
.login-card {
  max-width: 520px;
}

.login-hero {
  display: flex;
  gap: 14px;
  align-items: flex-start;
}

.login-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  background-color: var(--accent-soft);
  color: var(--accent);
}

.login-title {
  font-size: 15px;
  font-weight: 600;
  line-height: 1.6;
}

.login-sub {
  margin-top: 4px;
  font-size: 13px;
  color: var(--text-muted);
}
</style>
