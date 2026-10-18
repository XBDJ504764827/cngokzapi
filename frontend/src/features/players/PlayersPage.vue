<script setup lang="ts">
import { ref } from 'vue'
import { NAlert, NButton, NInput } from 'naive-ui'
import { useRouter } from 'vue-router'
import { resolvePlayer } from './api'

const router = useRouter()

const input = ref('')
const loading = ref(false)
const error = ref('')

async function submit(): Promise<void> {
  const q = input.value.trim()
  if (!q || loading.value) return
  loading.value = true
  error.value = ''
  try {
    const { steam_id64 } = await resolvePlayer(q)
    void router.push(`/profile/${steam_id64}`)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '查询失败，请检查输入'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="page-head">
    <h1 class="page-title">玩家查询</h1>
    <p class="page-sub">输入 SteamID、SteamID64 或 Steam 个人主页链接，查询玩家个人资料</p>
  </div>

  <div class="panel query-panel">
    <div class="panel-body">
      <div class="query-row">
        <NInput
          v-model:value="input"
          size="large"
          placeholder="76561198297415967 / STEAM_1:1:168575119 / steamcommunity.com/id/xxx"
          clearable
          @keyup.enter="submit"
        />
        <NButton type="primary" size="large" :loading="loading" @click="submit">查询</NButton>
      </div>

      <NAlert v-if="error" type="error" style="margin-top: 12px">{{ error }}</NAlert>

      <div class="formats">
        <div class="formats-title">支持以下格式</div>
        <ul>
          <li><span class="fmt">SteamID64</span> 17 位数字，如 <code>76561198297415967</code></li>
          <li><span class="fmt">SteamID</span> 如 <code>STEAM_1:1:168575119</code></li>
          <li><span class="fmt">SteamID3</span> 如 <code>[U:1:337150239]</code> 或纯数字账号 id</li>
          <li>
            <span class="fmt">个人主页</span>
            <code>steamcommunity.com/profiles/7656119…</code> 或
            <code>steamcommunity.com/id/自定义名</code>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<style scoped>
.query-panel {
  max-width: 860px;
}

.query-row {
  display: flex;
  gap: 10px;
}

.query-row :deep(.n-input) {
  flex: 1;
}

.formats {
  margin-top: 18px;
  padding-top: 14px;
  border-top: 1px dashed var(--border-color);
}

.formats-title {
  margin-bottom: 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.formats ul {
  display: flex;
  flex-direction: column;
  gap: 6px;
  list-style: none;
  font-size: 13px;
  color: var(--text-secondary);
}

.fmt {
  display: inline-block;
  min-width: 78px;
  font-weight: 600;
  color: var(--text-primary);
}

.formats code {
  padding: 1px 6px;
  border-radius: 4px;
  background-color: var(--bg-subtle);
  font-family: var(--font-mono);
  font-size: 12px;
}

@media (max-width: 600px) {
  .query-row {
    flex-direction: column;
  }
}
</style>
