# cngokz-frontend

Vue 3 + TypeScript + Vite + Naive UI。按业务模块组织，不按文件类型堆放。

## 目录

- `src/app/`：应用初始化（`main.ts`）、根组件、Naive UI 主题
- `src/router/`：路由及标题（权限守卫后续在此加）
- `src/layouts/`：页面布局（顶部导航 + 紧凑底部栏）
- `src/components/`：跨模块公共组件（暂无，有第二个使用者时再下沉）
- `src/features/`：业务模块，各含页面 + `api.ts` + `types.ts`
  - `leaderboard` / `bans` / `records`：已接真实接口（同名后端模块）
  - `home` / `servers` / `maps` / `profile` / `auth`：占位页，交互参考 `docs/prototype/cngokz-web`
- `src/services/http.ts`：全站唯一 fetch 出口
- `src/stores/`：Pinia（`app.currentMode` 为全站模式总开关）
- `src/composables/useApi.ts`：页面加载范式 `{ data, loading, error, reload }`
- `src/types/`：公共类型（JSON 一律 snake_case，与后端 models 一一对应）
- `src/utils/`：纯展示函数

## 开发

```bash
npm install
npm run dev      # http://<本机>:5173，/api 代理到 Axum（默认 127.0.0.1:3000）
npm run build    # 产物到 dist/，由后端托管（WEB_DIR=frontend/dist）
```

局域网预览：`http://192.168.0.101:5173`（Vite 已绑 0.0.0.0）。
联调后端：`VITE_API_PROXY=http://192.168.0.101:3000 npm run dev`。
