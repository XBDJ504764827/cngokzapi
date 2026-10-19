<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { NAlert, NAvatar, NButton, NInput, NSpin, NTag } from 'naive-ui'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useApi } from '@/composables/useApi'
import { MODES } from '@/types/common'
import type { Mode } from '@/types/common'
import { avatarUrl } from '@/utils/avatar'
import { countryName, flagUrl } from '@/utils/country'
import { formatNumber } from '@/utils/format'
import { mapImage } from '@/utils/mapImage'
import { resolvePlayer } from '@/features/players/api'
import { fetchProfile } from './api'
import { levelOf } from './level'
import type { PlayerScoreEntry } from './types'

const route = useRoute()
const router = useRouter()

const steamId = computed(() => (route.params.steam_id as string | undefined) ?? '')
const input = ref(steamId.value)
const queryError = ref('')
const resolving = ref(false)

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
  if (copyTimer) clearTimeout(copyTimer)
})

async function submit(): Promise<void> {
  const q = input.value.trim()
  if (!q || resolving.value) return
  queryError.value = ''
  resolving.value = true
  try {
    const { steam_id64 } = await resolvePlayer(q)
    void router.push(`/profile/${steam_id64}`)
  } catch (e) {
    queryError.value = e instanceof Error ? e.message : '查询失败，请检查输入'
  } finally {
    resolving.value = false
  }
}

// ---------- 基础资料（头像 / 国旗 / SteamID） ----------
const countryLabel = computed(() => countryName(data.value?.country))
const countryFlag = computed(() => flagUrl(data.value?.country))
/** 区域与国籍重复（如 region = CN）时不重复展示。 */
const regionLabel = computed(() => {
  const region = data.value?.region
  if (!region) return null
  if (data.value?.country && region.toUpperCase() === data.value.country.toUpperCase()) {
    return null
  }
  return region
})
const profileUrl = computed(() =>
  data.value ? `https://steamcommunity.com/profiles/${data.value.steam_id64}` : '',
)

const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | undefined

async function copySteamId(): Promise<void> {
  const id = data.value?.steam_id64
  if (!id) return
  try {
    await navigator.clipboard.writeText(id)
    copied.value = true
    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => (copied.value = false), 1500)
  } catch {
    /* 剪贴板不可用时静默失败 */
  }
}

/** 国旗图片加载失败时隐藏，只保留国家名文字。 */
function onFlagError(e: Event): void {
  ;(e.target as HTMLImageElement).style.display = 'none'
}

// ---------- 三模式档案 ----------
interface ModeCard {
  mode: Mode
  score: PlayerScoreEntry | null
  level: ReturnType<typeof levelOf>
}

const scores = computed(() => data.value?.scores ?? [])

/** 固定按 KZT / SKZ / VNL 顺序渲染，缺失的模式也占一张卡。 */
const modeCards = computed<ModeCard[]>(() => {
  const byMode = new Map(scores.value.map((s) => [s.mode, s]))
  return MODES.map((m: Mode) => {
    const score = byMode.get(m) ?? null
    return { mode: m, score, level: levelOf(score?.rating) }
  })
})

/** 最近成绩按模式分组（后端已按模式各取 20 条有分记录）。 */
const recentByMode = computed(() =>
  MODES.map((m: Mode) => ({
    mode: m,
    records: (data.value?.recent_records ?? []).filter((r) => r.mode === m),
  })),
)

/** 地图缩略图加载失败时隐藏，露出底色占位。 */
function onImageError(e: Event): void {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}
</script>

<template>
  <div class="page-head profile-head">
    <span class="page-eyebrow">PLAYER PROFILE · 玩家资料</span>
    <h1 class="page-title">个人资料</h1>
    <p class="page-sub">查询玩家三模式积分、全球排名与等级；首次查询会在后台从上游同步数据</p>
  </div>

  <div class="panel query-panel">
    <div class="query-inner">
      <div class="query-row">
        <NInput
          v-model:value="input"
          size="large"
          placeholder="SteamID / SteamID64 / Steam 个人主页链接"
          clearable
          @keyup.enter="submit"
        >
          <template #prefix>
            <svg
              class="query-icon"
              width="17"
              height="17"
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
        <NButton type="primary" size="large" :loading="resolving" class="query-btn" @click="submit">
          查询
        </NButton>
      </div>
      <NAlert v-if="queryError" type="error" class="query-alert">{{ queryError }}</NAlert>
      <p v-if="!steamId" class="query-hint">
        直接访问 <code class="mono">/profile/&lt;SteamID64&gt;</code>，或点击排行榜 / 封禁名单 /
        地图榜中的玩家名进入。
      </p>
    </div>
  </div>

  <NSpin v-if="steamId" :show="loading">
    <NAlert v-if="error" type="error" style="margin-bottom: 12px">{{ error }}</NAlert>
    <NAlert v-if="data?.syncing" type="info" style="margin-bottom: 12px">
      该玩家数据正在后台同步，页面稍后自动刷新（首次约需十几秒）。
    </NAlert>

    <!-- ================= 玩家身份卡 ================= -->
    <div v-if="data" class="profile-hero">
      <div class="hero-avatar-wrap">
        <NAvatar
          :src="avatarUrl(data.steam_id64, data.avatar)"
          :size="88"
          round
          class="profile-avatar"
        />
        <img
          v-if="countryFlag"
          :src="countryFlag"
          class="hero-flag"
          :alt="countryLabel ?? ''"
          :title="countryLabel ?? ''"
          @error="onFlagError"
        />
      </div>

      <div class="profile-info">
        <div class="profile-name">{{ data.name || '未知玩家' }}</div>
        <div class="profile-meta">
          <button
            type="button"
            class="steam-copy mono"
            :title="copied ? '已复制' : '复制 SteamID64'"
            @click="copySteamId"
          >
            <span>{{ data.steam_id64 }}</span>
            <svg
              v-if="!copied"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <rect x="9" y="9" width="11" height="11" rx="2" />
              <path d="M5 15V5a2 2 0 0 1 2-2h8" />
            </svg>
            <svg
              v-else
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
          </button>
          <span v-if="countryLabel" class="chip">{{ countryLabel }}</span>
          <span v-if="regionLabel" class="chip chip-accent">{{ regionLabel }} 区</span>
        </div>
      </div>

      <a class="steam-link" :href="profileUrl" target="_blank" rel="noopener noreferrer">
        Steam 主页
        <svg
          width="13"
          height="13"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M7 17 17 7M8 7h9v9" />
        </svg>
      </a>
    </div>

    <!-- ================= 三模式档案 ================= -->
    <section v-if="scores.length" class="section modes-section">
      <div class="section-header">
        <h2 class="section-title">模式档案</h2>
        <span class="section-note">积分 / 排名 / 等级均来自 GOKZ.TOP 口径</span>
      </div>

      <div class="mode-grid">
        <article
          v-for="card in modeCards"
          :key="card.mode"
          class="panel mode-card"
          :class="[`mode-${card.mode.toLowerCase()}`, { 'is-empty': !card.score }]"
        >
          <div class="mode-head">
            <span class="mode-badge">{{ card.mode }}</span>
            <span v-if="card.score && card.score.rank !== null" class="mode-rank">
              全球 <strong class="mono">#{{ card.score.rank }}</strong>
            </span>
            <span v-else class="mode-rank muted">未上榜</span>
          </div>

          <template v-if="card.score">
            <div class="level-block">
              <div class="level-line">
                <span class="level-name">{{ card.level?.name ?? '未评级' }}</span>
                <span class="level-rt mono">
                  {{
                    card.score.rating === null ? '—' : `Rt ${card.score.rating.toFixed(2)}`
                  }}
                </span>
              </div>
              <div
                class="level-meter"
                :aria-label="`等级 ${card.level?.index ?? 0} / 10`"
                role="img"
              >
                <span
                  v-for="i in 10"
                  :key="i"
                  class="level-seg"
                  :class="{ on: (card.level?.index ?? 0) >= i }"
                />
              </div>
              <div v-if="card.level && card.level.next && card.level.toNext !== null" class="level-next">
                升到 {{ card.level.next }} 还需 Rt {{ card.level.toNext.toFixed(2) }}
              </div>
            </div>

            <div class="points-block">
              <span class="points-label">全球总积分</span>
              <span class="points-value mono">{{ formatNumber(card.score.points) }}</span>
            </div>

            <dl class="mode-stats">
              <div>
                <dt>完成地图</dt>
                <dd class="mono">{{ formatNumber(card.score.maps_completed) }}</dd>
              </div>
              <div>
                <dt>WR NUB / PRO</dt>
                <dd class="mono">{{ card.score.wrs_nub }} / {{ card.score.wrs_pro }}</dd>
              </div>
              <div>
                <dt>本站已同步</dt>
                <dd class="mono">
                  {{
                    card.score.points_local === null
                      ? '—'
                      : formatNumber(card.score.points_local)
                  }}
                </dd>
              </div>
            </dl>
          </template>

          <div v-else class="mode-empty">该模式暂无榜单记录</div>
        </article>
      </div>
    </section>
    <NAlert v-else-if="data && !data.syncing" type="warning" style="margin-bottom: 12px">
      上游没有该玩家的榜单记录（可能未上榜或 SteamID64 有误）。
    </NAlert>

    <!-- ================= 最近成绩（保持不变） ================= -->
    <section class="section recent-section">
      <div class="section-header">
        <h2 class="section-title">最近成绩</h2>
        <span class="section-note">按模式分组 · 仅显示有分成绩 · 点击地图查看详情</span>
      </div>

      <div class="recent-columns">
        <div v-for="col in recentByMode" :key="col.mode" class="panel recent-col" :class="`mode-${col.mode.toLowerCase()}`">
          <div class="panel-head">
            <div class="rec-head">
              <span class="mode-dot" />
              {{ col.mode }}
            </div>
            <div class="panel-head-extra">
              <NTag type="success" size="small">共 {{ col.records.length }} 条</NTag>
            </div>
          </div>

          <div class="rec-list">
            <div
              v-for="r in col.records"
              :key="`${r.map_id}-${r.stage}-${r.time}-${r.date}`"
              class="rec-row"
            >
              <RouterLink
                class="rec-thumb"
                :to="`/maps/${r.map_id}`"
                :title="`查看 ${r.map} 详情`"
              >
                <div class="rec-thumb-fallback">
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="M9 3 3 6v15l6-3 6 3 6-3V3l-6 3-6-3Zm0 0v15m6-12v15" />
                  </svg>
                </div>
                <img
                  :src="mapImage(r.map, 'thumbnails')"
                  :alt="r.map"
                  loading="lazy"
                  @error="onImageError"
                />
              </RouterLink>

              <div class="rec-body">
                <div class="rec-line">
                  <RouterLink class="rec-map" :to="`/maps/${r.map_id}`" :title="r.map">
                    {{ r.map }}
                  </RouterLink>
                  <span class="rec-time mono">{{ r.time }}</span>
                </div>
                <div class="rec-line rec-meta">
                  <span>{{ r.stage === 0 ? '主关' : `B${r.stage}` }}</span>
                  <span class="sep">·</span>
                  <span class="rec-points mono">{{ r.points }} 分</span>
                  <span class="sep">·</span>
                  <span class="mono">{{ r.date }}</span>
                </div>
              </div>
            </div>

            <div v-if="!col.records.length" class="rec-empty">暂无有分成绩</div>
          </div>
        </div>
      </div>
    </section>

    <div class="back">
      <RouterLink to="/leaderboard">← 返回排行榜</RouterLink>
    </div>
  </NSpin>
</template>

<style scoped>
.profile-head {
  position: relative;
}

.profile-head::before {
  content: '';
  position: absolute;
  inset: -40px 0 auto 0;
  height: 180px;
  background: radial-gradient(520px 180px at 12% 45%, var(--accent-wash), transparent 70%);
  pointer-events: none;
  z-index: -1;
}

/* ---------- 查询卡 ---------- */
.query-panel {
  margin-bottom: 20px;
}

.query-inner {
  padding: 18px 20px;
}

.query-row {
  display: flex;
  gap: 10px;
  align-items: stretch;
}

.query-row :deep(.n-input) {
  flex: 1;
  min-width: 0;
}

.query-icon {
  color: var(--text-muted);
}

.query-btn {
  flex-shrink: 0;
  min-width: 96px;
  font-weight: 600;
}

.query-alert {
  margin-top: 12px;
}

.query-hint {
  margin-top: 12px;
  font-size: 13px;
  color: var(--text-muted);
}

.query-hint code {
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-subtle);
  font-family: var(--font-mono);
  font-size: 12px;
}

/* ---------- 玩家身份卡 ---------- */
.profile-hero {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 22px 24px;
  margin-bottom: 8px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-card);
}

.hero-avatar-wrap {
  position: relative;
  flex-shrink: 0;
}

.profile-avatar {
  border: 3px solid var(--accent-soft);
  background-color: var(--bg-subtle);
}

.hero-flag {
  position: absolute;
  right: -2px;
  bottom: -2px;
  width: 26px;
  height: 20px;
  object-fit: cover;
  border: 2px solid var(--bg-card);
  border-radius: 6px;
  box-shadow: var(--shadow-xs);
}

.profile-info {
  flex: 1;
  min-width: 0;
}

.profile-name {
  font-size: 26px;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1.25;
  margin-bottom: 10px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.profile-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

/* SteamID64 复制按钮：默认低调，hover 才强调 */
.steam-copy {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 9px;
  border: 1px solid transparent;
  border-radius: 999px;
  background-color: var(--bg-subtle);
  font-family: var(--font-mono);
  font-size: 12.5px;
  color: var(--text-secondary);
  cursor: pointer;
  transition:
    border-color 0.16s ease,
    color 0.16s ease,
    background-color 0.16s ease;
}

.steam-copy:hover {
  border-color: var(--accent);
  background-color: var(--accent-soft);
  color: var(--accent);
}

.steam-link {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 7px 14px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-md);
  background-color: var(--bg-card);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  text-decoration: none;
  transition:
    border-color 0.16s ease,
    color 0.16s ease,
    box-shadow 0.16s ease,
    transform 0.16s ease;
}

.steam-link:hover {
  transform: translateY(-1px);
  border-color: var(--accent);
  color: var(--accent);
  box-shadow: var(--shadow-sm);
}

/* ---------- 三模式档案 ---------- */
.modes-section {
  margin-top: 26px;
}

.mode-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
  align-items: stretch;
}

.mode-grid .panel + .panel {
  margin-top: 0;
}

.mode-card {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 18px;
  border-top: 3px solid var(--mode-color, var(--accent));
}

.mode-card.is-empty {
  opacity: 0.9;
}

.mode-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.mode-badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 12px;
  border-radius: 999px;
  background-color: var(--mode-color, var(--accent));
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: #fff;
}

.mode-rank {
  font-size: 12.5px;
  color: var(--text-muted);
}

.mode-rank strong {
  font-size: 15px;
  font-weight: 700;
  color: var(--mode-color, var(--text-primary));
}

.mode-rank.muted {
  color: var(--text-muted);
}

.level-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.level-line {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
}

.level-name {
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--mode-color, var(--accent));
}

.level-rt {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.level-meter {
  display: grid;
  grid-template-columns: repeat(10, minmax(0, 1fr));
  gap: 3px;
}

.level-seg {
  height: 6px;
  border-radius: 3px;
  background-color: var(--bg-hover);
}

.level-seg.on {
  background-color: var(--mode-color, var(--accent));
}

.level-next {
  font-size: 11.5px;
  color: var(--text-muted);
}

.points-block {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 14px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background-color: var(--mode-soft, var(--tint-primary));
}

.points-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.points-value {
  font-size: 19px;
  font-weight: 700;
  color: var(--text-primary);
}

.mode-stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  padding-top: 14px;
  border-top: 1px dashed var(--border-color);
}

.mode-stats dt {
  font-size: 11.5px;
  color: var(--text-muted);
}

.mode-stats dd {
  margin-top: 3px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.mode-empty {
  padding: 28px 8px;
  text-align: center;
  font-size: 13px;
  color: var(--text-muted);
}

/* ---------- 最近成绩（三模式并排，保持原样） ---------- */
.recent-section {
  margin-top: 26px;
}

.section-note {
  font-size: 12px;
  color: var(--text-muted);
}

.recent-columns {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
  align-items: start;
}

/* 同行面板取消全局「相邻面板上间距」，否则第 2/3 列会下沉 */
.recent-columns .panel + .panel {
  margin-top: 0;
}

.rec-head {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 700;
}

.mode-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--mode-color, var(--accent));
  box-shadow: 0 0 0 3px var(--mode-ring, var(--ring-accent));
}

.rec-list {
  display: flex;
  flex-direction: column;
}

.rec-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
}

.rec-row + .rec-row {
  border-top: 1px solid var(--border-color);
}

.rec-row:hover {
  background-color: var(--row-hover);
}

.rec-thumb {
  position: relative;
  flex-shrink: 0;
  width: 72px;
  height: 42px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-subtle);
  overflow: hidden;
  transition:
    box-shadow 0.15s ease,
    transform 0.15s ease;
}

.rec-thumb:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.rec-thumb-fallback {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.rec-thumb img {
  position: relative;
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.rec-body {
  flex: 1;
  min-width: 0;
}

.rec-line {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
}

.rec-line + .rec-line {
  margin-top: 2px;
}

.rec-map {
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.rec-map:hover {
  color: var(--accent);
  text-decoration: underline;
  text-underline-offset: 2px;
}

.rec-time {
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 700;
  color: var(--accent);
}

.rec-meta {
  justify-content: flex-start;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
}

.rec-meta .sep {
  color: var(--border-strong);
}

.rec-points {
  color: var(--accent-4);
  font-weight: 700;
}

.rec-empty {
  padding: 36px 16px;
  text-align: center;
  font-size: 13px;
  color: var(--text-muted);
}

.back {
  margin-top: 20px;
}

.back a {
  color: var(--accent);
  text-decoration: none;
  font-weight: 600;
}

@media (max-width: 1080px) {
  .mode-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .recent-columns {
    grid-template-columns: 1fr;
  }

  .recent-columns .panel + .panel {
    margin-top: 16px;
  }
}

@media (max-width: 720px) {
  .mode-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .profile-hero {
    flex-direction: column;
    align-items: flex-start;
    text-align: left;
  }

  .profile-name {
    white-space: normal;
  }

  .steam-link {
    align-self: stretch;
    justify-content: center;
  }
}

@media (max-width: 600px) {
  .query-row {
    flex-direction: column;
  }

  .query-btn {
    width: 100%;
  }

  .mode-stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
