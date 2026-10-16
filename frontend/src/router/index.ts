// 路由及标题。按业务模块懒加载；路由权限（Steam 登录后可见页）后续在此加守卫。
import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'home', component: () => import('@/features/home/HomePage.vue'), meta: { title: '首页' } },
  { path: '/leaderboard', name: 'leaderboard', component: () => import('@/features/leaderboard/LeaderboardPage.vue'), meta: { title: '排行榜' } },
  { path: '/maps', name: 'maps', component: () => import('@/features/maps/MapsPage.vue'), meta: { title: '地图' } },
  { path: '/servers', name: 'servers', component: () => import('@/features/servers/ServersPage.vue'), meta: { title: '服务器' } },
  { path: '/records', name: 'records', component: () => import('@/features/records/RecordsPage.vue'), meta: { title: '最新记录' } },
  { path: '/profile', name: 'profile', component: () => import('@/features/profile/ProfilePage.vue'), meta: { title: '个人资料' } },
  { path: '/bans', name: 'bans', component: () => import('@/features/bans/BansPage.vue'), meta: { title: '封禁名单' } },
  { path: '/login', name: 'login', component: () => import('@/features/auth/LoginPage.vue'), meta: { title: '登录' } },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.afterEach((to) => {
  const title = typeof to.meta.title === 'string' ? to.meta.title : ''
  document.title = title ? `${title} - CNGOKZ` : 'CNGOKZ - 中国 GOKZ 社区'
})

export default router
