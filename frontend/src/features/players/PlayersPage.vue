<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { NAlert, NAvatar, NButton, NInput, NTag } from 'naive-ui'
import { RouterLink, useRouter } from 'vue-router'
import { avatarUrl } from '@/utils/avatar'
import { resolvePlayer } from './api'
import { detectQuery } from './detect'
import type { QueryKind } from './detect'

const router = useRouter()

const RECENT_KEY = 'cngokz:recent-player-queries'
const MAX_RECENT = 6

interface RecentQuery {
  steam_id64: string
  input: string
  label: string
  at: number
}

const input = ref('')
const loading = ref(false)
const error = ref('')
const inputEl = ref<InstanceType<typeof NInput> | null>(null)
const recent = ref<RecentQuery[]>(loadRecent())

const detection = computed(() => detectQuery(input.value))
const trimmed = computed(() => input.value.trim())
const isUnknown = computed(() => trimmed.value.length > 0 && detection.value.kind === 'unknown')

const examples = [
  { label: 'SteamID64', value: '76561198297415967' },
  { label: 'STEAM_x:y:z', value: 'STEAM_1:1:168575119' },
  { label: 'SteamID3', value: '[U:1:337150239]' },
  { label: '主页链接', value: 'steamcommunity.com/id/gaben' },
]

const formats = [
  {
    name: 'SteamID64',
    desc: '17 位数字，最稳定的玩家唯一标识',
    example: '76561198297415967',
    icon: 'M4 9h16M4 15h16M10 3 8 21M16 3l-2 18',
  },
  {
    name: 'SteamID',
    desc: '经典 STEAM_x:y:z 写法',
    example: 'STEAM_1:1:168575119',
    icon: 'M12 3 4 7v10l8 4 8-4V7l-8-4Zm0 0v18M4 7l8 4 8-4',
  },
  {
    name: 'SteamID3',
    desc: '[U:1:w] 或纯数字账号 id',
    example: '[U:1:337150239]',
    icon: 'M7 8l-4 4 4 4m10-8 4 4-4 4M14 4l-4 16',
  },
  {
    name: '个人主页链接',
    desc: '/profiles/ 直读；/id/ 自定义名需联网解析',
    example: 'steamcommunity.com/id/gaben',
    icon: 'M10 13a5 5 0 0 0 7.07 0l1.42-1.42a5 5 0 0 0-7.07-7.07L10 6m4 5a5 5 0 0 0-7.07 0l-1.42 1.42a5 5 0 0 0 7.07 7.07L14 18',
  },
]

/** 读入本地保存的最近查询；解析失败时静默忽略（隐私模式等）。 */
function loadRecent(): RecentQuery[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY)
    if (!raw) return []
    const parsed: unknown = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed
      .filter(
        (item): item is RecentQuery =>
          typeof item === 'object' &&
          item !== null &&
          typeof (item as RecentQuery).steam_id64 === 'string' &&
          /^\d{17}$/.test((item as RecentQuery).steam_id64),
      )
      .slice(0, MAX_RECENT)
  } catch {
    return []
  }
}

function persistRecent(): void {
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recent.value))
  } catch {
    /* 存储不可用时忽略，不影响查询 */
  }
}

/** 最近查询里用短格式名，避免窄卡片把时间挤掉。 */
function shortLabel(kind: QueryKind): string {
  switch (kind) {
    case 'steamid64':
      return 'SteamID64'
    case 'steam2':
      return 'SteamID'
    case 'steam3':
    case 'account':
      return 'SteamID3'
    case 'profile':
    case 'vanity':
      return '主页链接'
    default:
      return '其他格式'
  }
}

/** 成功解析后把玩家放到最近查询首位（去重 + 截断）。 */
function remember(steamId64: string, query: string, label: string): void {
  const entry: RecentQuery = {
    steam_id64: steamId64,
    input: query,
    label,
    at: Date.now(),
  }
  recent.value = [entry, ...recent.value.filter((r) => r.steam_id64 !== steamId64)].slice(
    0,
    MAX_RECENT,
  )
  persistRecent()
}

function clearRecent(): void {
  recent.value = []
  persistRecent()
}

async function submit(): Promise<void> {
  const q = trimmed.value
  if (loading.value) return
  if (!q) {
    error.value = '请输入 SteamID、SteamID64 或 Steam 个人主页链接'
    inputEl.value?.focus()
    return
  }
  loading.value = true
  error.value = ''
  try {
    const { steam_id64 } = await resolvePlayer(q)
    remember(steam_id64, q, shortLabel(detection.value.kind))
    void router.push(`/profile/${steam_id64}`)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '查询失败，请检查输入'
  } finally {
    loading.value = false
  }
}

/** 点击示例 / 格式卡：填入并立即查询，减少一次点击。 */
function useExample(value: string): void {
  input.value = value
  error.value = ''
  void submit()
}

function formatRecentTime(ts: number): string {
  const d = new Date(ts)
  const pad = (n: number): string => String(n).padStart(2, '0')
  return `${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

onMounted(() => {
  // 桌面端聚焦搜索框方便直接输入；移动端不主动弹键盘
  if (window.matchMedia('(min-width: 720px)').matches) inputEl.value?.focus()
})
</script>

<template>
  <div class="page-head players-hero">
    <span class="page-eyebrow">PLAYER LOOKUP · 玩家检索</span>
    <h1 class="page-title">玩家查询</h1>
    <p class="page-sub">
      粘贴 SteamID64、SteamID、SteamID3 或 Steam 个人主页链接，一键进入玩家资料
    </p>
  </div>

  <div class="panel search-panel">
    <div class="search-inner">
      <div class="search-row">
        <NInput
          ref="inputEl"
          v-model:value="input"
          size="large"
          placeholder="76561198297415967 / STEAM_1:1:168575119 / steamcommunity.com/id/xxx"
          clearable
          :status="isUnknown ? 'warning' : undefined"
          @keyup.enter="submit"
        >
          <template #prefix>
            <svg
              class="search-icon"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <circle cx="11" cy="11" r="7" />
              <path d="m20 20-3.5-3.5" />
            </svg>
          </template>
        </NInput>
        <NButton type="primary" size="large" :loading="loading" class="search-btn" @click="submit">
          查询资料
        </NButton>
      </div>

      <div class="detect-line">
        <template v-if="!trimmed">
          <span class="detect-hint">支持 4 类写法 · 输入时自动识别格式</span>
        </template>
        <template v-else-if="isUnknown">
          <span class="detect-fail">
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              aria-hidden="true"
            >
              <circle cx="12" cy="12" r="9" />
              <path d="M12 8v4m0 4h.01" />
            </svg>
            无法识别该格式，请检查输入
          </span>
        </template>
        <template v-else>
          <span class="detect-ok">
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path d="m5 12 4.5 4.5L19 7" />
            </svg>
            已识别为 <strong>{{ detection.label }}</strong>
          </span>
          <span v-if="detection.steamId64" class="detect-preview">
            SteamID64 <code class="mono">{{ detection.steamId64 }}</code>
          </span>
          <span v-else class="detect-preview">提交后将联网解析自定义主页名</span>
        </template>
      </div>

      <NAlert v-if="error" type="error" class="search-alert" @close="error = ''">
        {{ error }}
      </NAlert>

      <div class="example-row">
        <span class="example-label">示例</span>
        <button
          v-for="ex in examples"
          :key="ex.value"
          type="button"
          class="example-chip"
          :title="`用 ${ex.label} 示例查询`"
          @click="useExample(ex.value)"
        >
          <span class="example-kind">{{ ex.label }}</span>
          <span class="mono example-value">{{ ex.value }}</span>
        </button>
      </div>
    </div>
  </div>

  <section v-if="recent.length" class="section recent-section">
    <div class="section-header">
      <h2 class="section-title">最近查询</h2>
      <NButton size="small" @click="clearRecent">清除记录</NButton>
    </div>
    <div class="recent-grid">
      <RouterLink
        v-for="r in recent"
        :key="r.steam_id64"
        :to="`/profile/${r.steam_id64}`"
        class="recent-item hover-lift"
        :title="`查看 ${r.steam_id64} 的资料`"
      >
        <NAvatar :src="avatarUrl(r.steam_id64)" :size="34" round class="recent-avatar" />
        <span class="recent-body">
          <span class="mono recent-id">{{ r.steam_id64 }}</span>
          <span class="recent-meta">
            {{ r.label }} · {{ formatRecentTime(r.at) }}
          </span>
        </span>
      </RouterLink>
    </div>
  </section>

  <div class="players-grid">
    <section class="panel formats-panel">
      <div class="panel-head">
        <div>
          <div class="panel-title">支持的输入格式</div>
          <div class="panel-sub">点击任意格式可直接用示例查询</div>
        </div>
        <NTag type="success" size="small" :bordered="false">自动识别</NTag>
      </div>
      <div class="panel-body">
        <div class="format-grid">
          <button
            v-for="f in formats"
            :key="f.name"
            type="button"
            class="format-card"
            @click="useExample(f.example)"
          >
            <span class="format-icon">
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <path :d="f.icon" />
              </svg>
            </span>
            <span class="format-body">
              <span class="format-name">{{ f.name }}</span>
              <span class="format-desc">{{ f.desc }}</span>
              <code class="format-example mono">{{ f.example }}</code>
            </span>
            <span class="format-go" aria-hidden="true">→</span>
          </button>
        </div>
      </div>
    </section>

    <aside class="panel tips-panel">
      <div class="panel-head">
        <div class="panel-title">如何找到自己的 SteamID</div>
      </div>
      <div class="panel-body">
        <ol class="tips">
          <li>
            <span class="tip-num">1</span>
            <span>打开 Steam 客户端，进入自己的<strong>个人资料</strong>页面。</span>
          </li>
          <li>
            <span class="tip-num">2</span>
            <span>复制页面上方地址栏里的个人资料网址。</span>
          </li>
          <li>
            <span class="tip-num">3</span>
            <span>把网址粘贴到上方搜索框，点击「查询资料」。</span>
          </li>
        </ol>
        <div class="tip-note">
          <svg
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <circle cx="12" cy="12" r="9" />
            <path d="M12 11v5m0-8.5h.01" />
          </svg>
          <span>
            游戏内控制台输入 <code class="mono">status</code> 也能看到自己的 SteamID；首次查询会从上游同步数据，
            通常需要十几秒。
          </span>
        </div>
      </div>
    </aside>
  </div>
</template>

<style scoped>
/* ---------- Hero：极浅墨绿晕染，保持全站白底简约 ---------- */
.players-hero {
  position: relative;
}

.players-hero::before {
  content: '';
  position: absolute;
  /* 左右不留负边距，避免窄屏产生横向滚动 */
  inset: -40px 0 auto 0;
  height: 200px;
  background: radial-gradient(520px 190px at 12% 45%, var(--accent-wash), transparent 70%);
  pointer-events: none;
  z-index: -1;
}

/* ---------- 搜索卡 ---------- */
.search-panel {
  position: relative;
  background-image: radial-gradient(680px 240px at 0% 0%, var(--accent-wash), transparent 72%);
}

.search-panel::before {
  content: '';
  position: absolute;
  inset: 0 0 auto 0;
  height: 3px;
  background-image: linear-gradient(90deg, var(--accent), var(--accent-soft) 55%, transparent);
}

.search-inner {
  padding: 22px;
}

.search-row {
  display: flex;
  gap: 10px;
  align-items: stretch;
}

.search-row :deep(.n-input) {
  flex: 1;
  min-width: 0;
  background-color: var(--bg-card);
}

.search-row :deep(.n-input .n-input__border),
.search-row :deep(.n-input .n-input__state-border) {
  border-radius: var(--radius-md);
}

.search-icon {
  color: var(--text-muted);
}

.search-btn {
  flex-shrink: 0;
  min-width: 120px;
  font-weight: 600;
}

/* 实时格式识别 */
.detect-line {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  min-height: 26px;
  margin-top: 10px;
  font-size: 12.5px;
}

.detect-hint {
  color: var(--text-muted);
}

.detect-ok,
.detect-fail {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-weight: 500;
}

.detect-ok {
  color: var(--accent);
}

.detect-ok strong {
  font-weight: 700;
}

.detect-fail {
  color: var(--danger);
}

.detect-preview {
  color: var(--text-muted);
}

.detect-preview code {
  margin-left: 4px;
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-subtle);
  color: var(--text-secondary);
  font-size: 12px;
}

.search-alert {
  margin-top: 12px;
}

/* 示例 chips */
.example-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px dashed var(--border-color);
}

.example-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.example-chip {
  display: inline-flex;
  align-items: baseline;
  gap: 7px;
  padding: 5px 10px;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  background-color: var(--bg-card);
  font-family: inherit;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  transition:
    border-color 0.16s ease,
    background-color 0.16s ease,
    color 0.16s ease,
    box-shadow 0.16s ease;
}

.example-chip:hover {
  border-color: var(--accent);
  background-color: var(--accent-soft);
  color: var(--accent);
  box-shadow: var(--shadow-xs);
}

.example-kind {
  font-weight: 700;
  color: var(--text-primary);
}

.example-chip:hover .example-kind {
  color: var(--accent);
}

.example-value {
  font-size: 11.5px;
  color: var(--text-muted);
}

.example-chip:hover .example-value {
  color: var(--accent);
}

/* ---------- 最近查询 ---------- */
.recent-section {
  margin-top: 26px;
}

.recent-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 11px 13px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-card);
  background-color: var(--bg-card);
  box-shadow: var(--shadow-sm);
  text-decoration: none;
  color: inherit;
}

.recent-avatar {
  flex-shrink: 0;
  border: 1px solid var(--border-color);
  background-color: var(--bg-subtle);
}

.recent-body {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.recent-id {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.recent-item:hover .recent-id {
  color: var(--accent);
}

.recent-meta {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ---------- 格式说明 + 使用提示 ---------- */
.players-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.9fr) minmax(0, 1fr);
  gap: 16px;
  align-items: start;
  margin-top: 26px;
}

/* 并排面板取消全局「相邻面板上间距」 */
.players-grid .panel + .panel {
  margin-top: 0;
}

.format-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.format-card {
  display: flex;
  align-items: flex-start;
  gap: 11px;
  width: 100%;
  padding: 14px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background-color: var(--bg-card);
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    transform 0.18s ease,
    border-color 0.18s ease,
    box-shadow 0.18s ease;
}

.format-card:hover {
  transform: translateY(-2px);
  border-color: var(--accent);
  box-shadow: var(--shadow-md);
}

.format-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 34px;
  height: 34px;
  border-radius: var(--radius-sm);
  background-color: var(--accent-soft);
  color: var(--accent);
}

.format-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.format-name {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
}

.format-desc {
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-muted);
}

.format-example {
  align-self: flex-start;
  max-width: 100%;
  margin-top: 4px;
  padding: 2px 7px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-subtle);
  font-size: 11.5px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.format-go {
  margin-left: auto;
  align-self: center;
  flex-shrink: 0;
  font-size: 14px;
  color: var(--border-strong);
  transition:
    color 0.16s ease,
    transform 0.16s ease;
}

.format-card:hover .format-go {
  color: var(--accent);
  transform: translateX(2px);
}

/* 使用提示 */
.tips {
  display: flex;
  flex-direction: column;
  gap: 12px;
  list-style: none;
  font-size: 13px;
  line-height: 1.55;
  color: var(--text-secondary);
}

.tips li {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.tips strong {
  color: var(--text-primary);
  font-weight: 600;
}

.tip-num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background-color: var(--accent-soft);
  color: var(--accent);
  font-size: 11px;
  font-weight: 700;
}

.tip-note {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  margin-top: 16px;
  padding: 11px 12px;
  border: 1px dashed var(--border-color);
  border-radius: var(--radius-md);
  background-color: var(--bg-subtle);
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-muted);
}

.tip-note svg {
  flex-shrink: 0;
  margin-top: 2px;
  color: var(--accent);
}

.tip-note code {
  padding: 1px 5px;
  border-radius: 4px;
  background-color: var(--bg-card);
  color: var(--text-secondary);
  font-size: 11.5px;
}

@media (max-width: 980px) {
  .players-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (max-width: 600px) {
  .search-inner {
    padding: 16px;
  }

  .search-row {
    flex-direction: column;
  }

  .search-btn {
    width: 100%;
  }

  .format-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .recent-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .example-row {
    align-items: flex-start;
  }

  .example-chip {
    max-width: 100%;
  }

  .example-value {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}
</style>
