<script setup lang="ts">
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { useAppStore } from '@/stores/app'

const app = useAppStore()
const router = useRouter()
const route = useRoute()

const links = [
  { to: '/', label: '首页' },
  { to: '/leaderboard', label: '排行榜' },
  { to: '/maps', label: '地图' },
  { to: '/servers', label: '服务器' },
  { to: '/records', label: '最新记录' },
  { to: '/bans', label: '封禁名单' },
]

function goLogin(): void {
  void router.push('/login')
}
</script>

<template>
  <header class="navbar">
    <div class="container nav-container">
      <RouterLink to="/" class="nav-brand">
        <span class="logo-mark">KZ</span>
        <span class="logo-texts">
          <span class="logo">CNGOKZ</span>
          <span class="logo-subtitle">中国 GOKZ 社区</span>
        </span>
      </RouterLink>
      <nav class="nav-links">
        <RouterLink
          v-for="l in links"
          :key="l.to"
          :to="l.to"
          class="nav-link"
          active-class="active"
          exact-active-class="active"
        >
          {{ l.label }}
        </RouterLink>
      </nav>
      <div class="nav-actions">
        <button class="mode-switch" :title="`当前${app.currentMode}，点击切换`" @click="app.cycleMode()">
          <span class="mode-dot" />
          {{ app.currentMode }}
        </button>
        <button class="btn-steam" @click="goLogin()">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path
              d="M12 2a10 10 0 0 1 10 10 10 10 0 0 1-10 10A10 10 0 0 1 2 12 10 10 0 0 1 12 2m0 18c4.41 0 8-3.59 8-8s-3.59-8-8-8-8 3.59-8 8 3.59 8 8 8m-2-13.5a2.5 2.5 0 0 1 2.5 2.5 2.5 2.5 0 0 1-2.5 2.5A2.5 2.5 0 0 1 7.5 9a2.5 2.5 0 0 1 2.5-2.5m5 5a2.5 2.5 0 0 1 2.5 2.5 2.5 2.5 0 0 1-2.5 2.5A2.5 2.5 0 0 1 12.5 14a2.5 2.5 0 0 1 2.5-2.5z"
            />
          </svg>
          Steam 登录
        </button>
      </div>
    </div>
  </header>

  <main class="main-content">
    <div class="container">
      <!-- key 到 path：路由切换时重挂载触发一次极轻的淡入 -->
      <div :key="route.path" class="route-view fade-in">
        <RouterView />
      </div>
    </div>
  </main>

  <footer class="footer">
    <div class="container footer-inner">
      <span class="footer-brand">CNGOKZ · 中国 GOKZ 社区</span>
      <span class="footer-links">
        <a href="#">关于我们</a>
        <a href="#">规则说明</a>
        <a href="#">API 文档</a>
      </span>
      <span class="footer-copy">&copy; 2024 CNGOKZ. All rights reserved.</span>
    </div>
  </footer>
</template>

<style scoped>
.navbar {
  background-color: rgba(255, 255, 255, 0.88);
  backdrop-filter: saturate(180%) blur(10px);
  border-bottom: 1px solid var(--border-color);
  box-shadow: var(--shadow-sm);
  position: sticky;
  top: 0;
  z-index: 100;
}

.nav-container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  min-height: 64px;
}

.nav-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  text-decoration: none;
}

.logo-mark {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  background-color: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  letter-spacing: -0.02em;
  box-shadow: 0 1px 2px rgba(29, 74, 53, 0.24);
}

.logo-texts {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.logo {
  font-size: 19px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--text-primary);
}

.logo-subtitle {
  font-size: 12px;
  color: var(--text-muted);
}

.nav-links {
  display: flex;
  gap: 2px;
  flex-wrap: nowrap;
  overflow-x: auto;
  min-width: 0;
  scrollbar-width: none;
}

.nav-links::-webkit-scrollbar {
  display: none;
}

.nav-link {
  padding: 7px 13px;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  text-decoration: none;
  white-space: nowrap;
  flex-shrink: 0;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.nav-link:hover {
  background-color: var(--bg-subtle);
  color: var(--text-primary);
}

.nav-link.active {
  background-color: var(--accent);
  color: #fff;
  font-weight: 600;
  box-shadow: 0 2px 8px rgba(47, 111, 79, 0.24);
}

.nav-link.active:hover {
  background-color: var(--accent-hover);
  color: #fff;
}

.nav-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.mode-switch {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 76px;
  padding: 7px 12px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.02em;
  color: var(--accent);
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease;
}

.mode-switch:hover {
  border-color: var(--accent);
  background-color: var(--accent-soft);
}

.mode-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(47, 111, 79, 0.14);
}

.btn-steam {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background-color: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font-base);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.btn-steam:hover {
  background-color: var(--accent-hover);
}

.main-content {
  padding: 36px 0 48px;
  min-height: calc(100vh - 64px - 53px);
}

.footer {
  background-color: var(--bg-card);
  border-top: 1px solid var(--border-color);
  padding: 12px 0;
}

.footer-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  font-size: 12px;
  color: var(--text-muted);
}

.footer-brand {
  font-weight: 700;
  color: var(--text-secondary);
}

.footer-links {
  display: flex;
  gap: 14px;
}

.footer-links a {
  color: var(--text-secondary);
  text-decoration: none;
}

.footer-links a:hover {
  color: var(--accent);
}

@media (max-width: 1100px) {
  .logo-subtitle {
    display: none;
  }
}

@media (max-width: 860px) {
  .nav-container {
    flex-wrap: wrap;
    padding-top: 8px;
    padding-bottom: 8px;
    row-gap: 6px;
  }

  .nav-links {
    order: 3;
    width: 100%;
  }
}

/* 窄屏：隐藏 Steam 按钮文字只留图标，保证导航链接不换行、不重叠 */
@media (max-width: 600px) {
  .btn-steam {
    padding: 8px 10px;
    font-size: 0;
    gap: 0;
  }

  .footer-inner {
    justify-content: center;
    text-align: center;
  }
}
</style>
