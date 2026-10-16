# 项目规范与技术设计（PROJECT）

> 记录项目应该如何运作：整体目标、功能需求、技术架构、目录结构、
> 数据库设计、API 规范、编码规范和开发流程。
> AI 开发前必须先读本文件与 CHANGELOG.md，开发后必须同步更新两者。

## 1. 项目整体目标

为中国 GOKZ 社区（CNGOKZ）提供后端 API + 官方网站。正式技术栈：

- 前端：Vue 3 + TypeScript + Vite，组件库 Naive UI（主题覆写为墨绿主色）；
- 后端：Rust + Axum 模块化单体；
- 数据库：PostgreSQL；缓存：Redis。

旧 vanilla 原型已归档到 `docs/prototype/cngokz-web`，仅作视觉与交互参照，
新功能以 Vue 实现为准，不再回写原型。

## 2. 功能需求（以原型 + 已上线页面为准）

原型位置：`docs/prototype/cngokz-web/`（只读参照）；Vue 实现位置：`frontend/src/features/`：

1. 首页：社区统计、在线服务器列表、最新记录。
2. 排行榜：全球玩家排名（KZT / SKZ / VNL 各 120 人，独立计分排序），
   每行含头像（模拟图，后续换 Steam 真实头像）、名称、SteamID64、完成地图、
   总分、平均分、记录数、最近游玩时间；支持名称模糊 / SteamID64 子串搜索、
   活跃度筛选（近 7 / 30 天、超 30 天未玩）、多字段排序，计数以标题右侧徽章展示。
   条目字段后续接真实数据时保持兼容（后端 `modules/leaderboard/repository.rs` 同口径）。
3. 地图列表：浏览全部地图，支持难度（Tier 1-8）筛选与搜索。
4. 地图详情：单个地图信息 + 前 100 名排行。
5. 服务器列表：全部在线服务器详细信息（地图、在线玩家、人数），默认列式展示，
   卡片不直接展示地址（仅复制 IP 按钮可用，防止爬虫抓取）。
6. 个人资料：玩家统计、完成地图、个人记录。
7. Steam 登录：当前为模拟，需接 Steam OpenID 真实登录。
8. 封禁名单（新增）：封禁玩家公示，每行含头像、名称、SteamID64、封禁理由、
   封禁时间、封禁服务器、详细数据（地图·模式·异常成绩·Demo 编号）；支持名称模糊 /
   SteamID64 子串搜索，每页 20 人翻页；当前 45 条模拟占位，真实管理数据接入时字段保持兼容。
9. 最新记录：KZT / SKZ / VNL 三列并排，各列最新 20 条，不再跟随导航栏模式开关。

设计约束（必须保留）：

- 浅色简约主题，白底 + 单一墨绿主色（Naive UI 主题覆写 `#2F6F4F`），细边框代替强阴影/发光。
- 克制干净，不做科技感/竞技风。
- 响应式布局（基础支持），信息层级清晰。
- 模式：KZT / SKZ / VNL；字体：Inter。
- 底部栏保持紧凑（内边距与间距已压薄，不得随意加高）。
- 顶部导航栏：链接永不断行，窄屏横向滚动或换两行，
  不得出现文字重叠；新增导航项时同步验证 1100 / 860 / 600 三档断点。

待后端支持的能力（当前为模拟数据）：

- Steam OpenID 登录、实时服务器数据、玩家记录查询、地图库、
  排行榜实时更新、玩家搜索、管理员功能（删记录、封禁玩家）。

## 3. 技术架构

### 3.1 选型

| 层 | 选型 | 为什么 |
|---|---|---|
| 前端 | Vue 3 + TypeScript + Vite + Naive UI + Pinia + vue-router | 主流组合，TS 类型与后端 models 一一对应；Naive UI 表格/分页/表单开箱即用，主题可覆写 |
| 后端 | Rust + Axum + Tokio + SQLx + tracing | 性能好、内存安全；Axum/Tokio 为 Rust 主流 Web 组合 |
| 数据库 | PostgreSQL | 排行、前 100、分页查询需要强一致 SQL 与索引；`backend/migrations/` 版本化管理 |
| 缓存 | Redis | 榜单等读多写少接口缓存（TTL 见各模块 service），无库也可启动（懒连接） |

### 3.2 运行形态

开发期（前后端分离，Vite 代理）：

```text
浏览器 :5173
  └─> Vite dev（frontend/）
        ├─ 页面/静态 -> 本地
        └─ /api/* ---> 127.0.0.1:3000（Axum，可用 VITE_API_PROXY 改目标）
```

生产（同源，Axum 托管构建产物）：

```text
浏览器
  └─> Axum（backend/）
        ├─ /, /assets/* -> frontend/dist（WEB_DIR 可覆盖，缺失只告警）
        └─ /api/v1/*    -> JSON API
```

### 3.3 后端分层（职责固定）

```text
backend/src/
├── main.rs            # 入口：日志 → 配置 → 状态 → 装配 → 监听
├── app.rs             # 路由装配（/api/v1 嵌套 + 静态 fallback + CORS/Trace）
├── config.rs          # 配置（环境变量，见 .env.example）
├── state.rs           # 共享状态（AppState：config/db/redis，直接 Clone）
├── errors.rs          # 统一错误 AppError → { error: { code, message } }
├── middleware/        # cors（开发放行）、后续加 auth/限流
├── infrastructure/    # db（PG 懒连接池）、cache（Redis Client），不写业务 SQL
└── modules/<domain>/  # 业务模块，六件套（缺省可省）：
    ├── routes.rs      # 只声明路径映射
    ├── handlers.rs    # 薄层：参数萃取 + 错误映射
    ├── service.rs     # 编排：校验、缓存策略、分页
    ├── repository.rs  # 唯一可碰 DB/外部数据处（占位期返回模拟数据）
    ├── models.rs      # 领域模型（Serialize）
    └── dto.rs         # 查询/提交对象（Deserialize）
```

规则：handler 不拼 SQL、不做业务判断；service 不碰传输细节；
repository 之外不得出现 `sqlx::query`；无数据访问的模块可省略 `repository.rs`。

### 3.4 前端分层（职责固定）

```text
frontend/src/
├── app/          # 初始化（main.ts）、根组件、Naive UI 主题
├── router/       # 路由及标题（权限守卫后续在此加）
├── layouts/      # 页面布局（导航 + 紧凑底部栏）
├── components/   # 跨模块公共组件（有第二个使用者时再下沉，勿提前抽象）
├── features/<domain>/  # 业务模块：页面 + api.ts + types.ts
├── services/http.ts     # 全站唯一 fetch 出口（错误体归一为 ApiError）
├── stores/       # Pinia（app.currentMode 为全站模式总开关）
├── composables/  # useApi（{ data, loading, error, reload } 范式）
├── types/        # 公共类型（JSON 一律 snake_case，与后端 models 一一对应）
└── utils/        # 纯展示函数
```

规则：页面只经 `features/<domain>/api.ts` 调接口；类型不做驼峰映射；
公共组件有两个使用者才下沉到 `components/`。

## 4. 目录结构（现状）

```text
cngokzapi/
├── frontend/             # Vue 3 + TS + Vite（见 frontend/README.md）
│   ├── src/
│   │   ├── app/          # 初始化、根组件、主题
│   │   ├── router/       # 路由及标题
│   │   ├── layouts/      # 页面布局
│   │   ├── components/   # 跨模块公共组件
│   │   ├── features/     # home/leaderboard/records/servers/maps/profile/bans/auth
│   │   ├── services/     # http.ts 全站唯一 fetch 出口
│   │   ├── stores/       # Pinia（app.currentMode）
│   │   ├── composables/  # useApi
│   │   ├── types/        # 公共类型（snake_case）
│   │   └── utils/        # 纯展示函数
│   ├── tests/            #（vitest 规划中）
│   ├── package.json
│   └── vite.config.ts    # /api 代理到 Axum
├── backend/              # Rust + Axum（见 backend/README.md）
│   ├── src/              # main/app/config/state/errors/middleware/infrastructure/modules
│   ├── migrations/       # 0001_init.sql（sqlx migrate 管理）
│   ├── tests/            #（集成测试规划中，单测在各模块内）
│   ├── Cargo.toml
│   └── Cargo.lock
├── docs/                 # 与 frontend/backend 同级
│   └── prototype/cngokz-web/  # 旧 vanilla 原型（只读参照）
├── scripts/              # 运维脚本（预留）
├── PROJECT.md
├── CHANGELOG.md
├── README.md
├── .env.example
└── Cargo.toml            # workspace（成员：backend）
```

## 5. 数据库设计（以 `backend/migrations/0001_init.sql` 为准）

- `players(steam_id PK, name, avatar_url, points, created_at, banned_until)`
- `maps(id PK, name UNIQUE, tier 1-8, mode, created_at)`
- `records(id PK, player_steam_id FK, map_id FK, mode, time_ms, points, created_at)`
  - 索引：`(map_id, mode, time_ms)` 用于地图前 100；`(player_steam_id, created_at)` 用于个人记录。
- `servers(id PK, name, map, ip, port, player_count, max_players, updated_at)`
- `bans(steam_id PK FK, name, avatar_url, reason, server, detail, banned_at)`
  - 索引：`(banned_at DESC)` 用于封禁名单倒序。
- `admins(steam_id PK, role, created_at)`

约定：全部时间 `TIMESTAMPTZ`（UTC）；成绩存毫秒整数 `time_ms`；积分用整数；
JSON 一律 snake_case，与前后端类型一一对应。

## 6. API 规范

- 前缀：`/api/v1`，JSON，UTF-8。已落地：
  `GET /api/v1/health`、`GET /api/v1/leaderboard?mode=KZT`、
  `GET /api/v1/records`、`GET /api/v1/bans?query=&page=&per_page=`。
- 命名：复数资源小写下划线，如 `GET /api/v1/players/:steam_id`、`GET /api/v1/maps`、
  `GET /api/v1/maps/:id/records?mode=KZT`、`GET /api/v1/servers`。
- 分页：`?page=1&per_page=20`（默认 20，最大 100），返回 `{ data, pagination: { page, per_page, total } }`。
- 错误：统一 `{ error: { code, message } }`，HTTP 状态码语义正确
  （400 参数错 / 401 未登录 / 403 无权限 / 404 不存在 / 500 服务错 / 503 依赖未就绪），`code` 如 `INVALID_PARAM`。
- 权限：普通接口公开；写操作与管理接口需 Steam 登录 + session/cookie 校验；
  管理员接口再校验 `admins` 表角色。
- 日志：Axum `TraceLayer` 记录请求方法、路径、状态码、耗时；业务错误记 `warn`，系统错误记 `error`。

## 7. 编码规范

- Rust：`cargo fmt` + `cargo clippy -D warnings` 必须通过；按 §3.3 分层，
  DB 查询一律在 `repository.rs`，不在 handler 里拼 SQL 字符串；
  `#[allow(dead_code)]` 必须写清消除条件，随首个消费者落地消除。
- Vue/TS：`npm run build`（含 `vue-tsc -b`）必须通过；`noUnusedLocals` 下无用引入直接删；
  类型 snake_case 不映射；注释用中文写清原因（复杂逻辑、Steam 流程、占位替换点）。
- 提交信息：`feat/fix/docs/chore: 简述`，中文，关联页面或 API。

## 8. 开发流程

1. 日常从 `develop` 切功能分支：`feat/xxx` 等。
2. 自测：后端 `cargo test` + `cargo clippy --all-targets -- -D warnings`；
   前端 `npm run build`（含 `vue-tsc -b`）；联调时后端 `cargo run` + 前端 `npm run dev` 走代理验证。
3. 合入 `develop` 前更新 `CHANGELOG.md`（按模板写清文件/DB/API/测试）。
4. 稳定后 `develop` -> `main`，打 tag。
5. AI 开发前必须检查：
   - 读 `PROJECT.md` + `CHANGELOG.md` 最新状态。
   - 对照 `docs/prototype/cngokz-web` 确认页面与样式（只读，不回写）。
   - 确认当前分支（日常在 `develop` 或功能分支，不直接提交 `main`）。
6. AI 开发后必须更新：
   - `CHANGELOG.md` 新增当日条目。
   - 若架构/目录/API/DB 有变，同步改 `PROJECT.md`。
   - 无 PG/Redis 时不得阻塞启动（懒连接），占位 `allow(dead_code)` 写清消除条件。

## 9. 当前状态与下一步

- 当前：第二步完成（2026-10-16，`develop`）：`web/` 三件套与 `~/cngokz-web` 逐字节一致，
  Axum `ServeDir` 托管（`/` 自动返回 `index.html`，`WEB_DIR` 可覆盖），
  `GET /api/v1/health` 与 `cargo test/clippy` 均通过。`web/` 已无占位。
  追加补丁：`HOST` 默认改为 `0.0.0.0`，开发机 192.168.0.101 上局域网可直接访问（`127.0.0.1` 仍可用环境变量切回）。
- 当前：排行榜加厚完成（2026-10-16，`develop`）：每模式 120 人、头像（模拟图）、
  SteamID64、最近游玩时间、名称/SteamID64 搜索 + 活跃度筛选 + 多字段排序，
  导航栏搜索框联动定位到排行榜；`cargo test/clippy` 与静态托管验证均通过。
- 当前：正式结构落地（2026-10-16，`develop`）：前端 Vue 3 + TS + Vite + Naive UI
 （8 路由，排行榜/封禁/记录三页接真实接口，其余占位），后端 Axum 模块化单体
  （health/leaderboard/records/bans 四模块，统一错误体，PG/Redis 懒连接），
  `backend/migrations/0001_init.sql` 落地；`cargo test` 8 passed、`clippy`、`vue-tsc` 构建全过，
  Vite 代理与 dist 托管联调验证通过。
- 下一步（按序）：
  1. ~~旧阶段（vanilla 原型 + 单文件后端）~~ ✅ 已归档/替换
  2. 剩余页面接真实接口：servers / maps / profile / home（后端 modules + 前端 features 对齐落地）
  3. Steam OpenID 登录 + 管理接口（删记录、封禁）+ Redis 缓存与 PG 回源替换模拟层
