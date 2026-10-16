<script setup lang="ts">
import { RouterLink, RouterView, useRouter } from 'vue-router'
import { useAppStore } from '@/stores/app'

const app = useAppStore()
const router = useRouter()

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
      <div class="nav-brand">
        <span class="logo">CNGOKZ</span>
        <span class="logo-subtitle">中国 GOKZ 社区</span>
      </div>
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
          {{ app.currentMode }}
        </button>
        <button class="btn-steam" @click="goLogin()">Steam 登录</button>
      </div>
    </div>
  </header>

  <main class="main-content">
    <div class="container">
      <RouterView />
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
  background-color: var(--bg-card);
  border-bottom: 1px solid var(--border-color);
  position: sticky;
  top: 0;
  z-index: 100;
}

.nav-container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  min-height: 60px;
}

.nav-brand {
  display: flex;
  align-items: baseline;
  gap: 10px;
  flex-shrink: 0;
}

.logo {
  font-size: 20px;
  font-weight: 700;
  color: var(--accent);
}

.logo-subtitle {
  font-size: 13px;
  color: var(--text-muted);
}

.nav-links {
  display: flex;
  gap: 4px;
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
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  text-decoration: none;
  white-space: nowrap;
  flex-shrink: 0;
}

.nav-link:hover {
  background-color: var(--bg-subtle);
  color: var(--text-primary);
}

.nav-link.active {
  background-color: var(--bg-subtle);
  color: var(--accent);
}

.nav-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.mode-switch {
  min-width: 52px;
  padding: 7px 12px;
  background-color: var(--bg-card);
  border: 1px solid var(--border-strong);
  border-radius: 10px;
  font-size: 13px;
  font-weight: 700;
  color: var(--accent);
  cursor: pointer;
}

.btn-steam {
  padding: 8px 14px;
  background-color: var(--accent);
  color: #fff;
  border: none;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.main-content {
  padding: 32px 0;
  min-height: calc(100vh - 60px - 52px);
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
</style>
