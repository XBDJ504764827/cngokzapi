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

数据来源与计分（2026-10-17 定稿，详见 §9）：历史成绩从 GlobalAPI 生态
（`api.gokz.top` 为主、`kztimerglobal.com` 为辅）**按需导入**，新成绩由自建
`cngokzapi` 服务器插件上报；**计分完全由本站独立完成**（标准 GOKZ 公式），
不采用、不回写外部站点的分数，多个站点之间互不影响。

## 2. 功能需求（以原型 + 已上线页面为准）

原型位置：`docs/prototype/cngokz-web/`（只读参照）；Vue 实现位置：`frontend/src/features/`：

1. 首页：社区统计（4 个统计卡，真实数据）、「关于 CNGOKZ」介绍卡、「最新公告」列表
   （位置＝介绍卡之后；当前为静态内容，见 `frontend/src/features/home/announcements.ts`）。
2. 排行榜：**KZT / SKZ / VNL 三模式同页并排**（各约 200 人，独立计分；每列展示前 50 名），
   每行含名次徽章、头像、名称（链接到资料页）、总分、完成地图、记录数、平均分、最近游玩；
   支持名称模糊 / SteamID64 子串搜索、活跃度筛选（近 7 / 30 天、超 30 天未玩）、多字段排序，
   筛选与排序同时作用于三列；**导航栏不再提供模式切换**。
   条目字段后续接真实数据时保持兼容（后端 `modules/leaderboard/repository.rs` 同口径）。
3. 地图列表：浏览全部地图，支持难度（按当前模式 Tier）筛选与名称搜索；
   **卡片网格排列**（仿 gokz.top，宽屏默认 4 列，窄屏降为 3 / 2 / 1 列）：地图图片、
   地图名称、制作人、难度徽章、是否有奖励关（Bonus ×N / 无 Bonus），
   并显示当前模式主关 WR（记录人 + 耗时 + 日期）；每页 24 张分页。图片走 `utils/mapImage.ts`
   统一出口（当前用 KZGlobalTeam 公开图库，后续可换自建图床）。
4. 地图详情：单个地图信息 + 前 100 名排行。
5. 服务器列表：全部在线服务器详细信息，提供**两种视图切换**——卡片排列（grid，**默认**）与
   列状排列（list）；两种视图都显示当前地图图片、地图名、难度与区域、人数、社区标签、
   在线玩家昵称标签。支持**按社区分类筛选**（后端 `servers/category.rs` 归一化：优先 `group_name`，
   缺失时从服务器名推导，如 CNGOKZ / AXE / HIME；分类栏计数倒序）。
   地址一律不渲染明文：提供「复制 IP」与「加入游戏」两个按钮，后者点击时用
   `steam://rungameid/<CS_APP_ID>//+connect ip:port` 拉起 Steam 客户端
   （`CS_APP_ID` 常量默认 `4465480`，集中在前端 `ServersPage.vue`，便于后续更换）。
6. 个人资料：玩家统计、完成地图、个人记录。
7. Steam 登录：当前为模拟，需接 Steam OpenID 真实登录。
8. 封禁名单（新增）：封禁玩家公示，每行含头像、名称、SteamID64、封禁理由、
   封禁时间、封禁服务器、详细数据（地图·模式·异常成绩·Demo 编号）；支持名称模糊 /
   SteamID64 子串搜索，每页 20 人翻页；当前 45 条模拟占位，真实管理数据接入时字段保持兼容。
9. 最新记录：KZT / SKZ / VNL 三列并排，各列最新 20 条（不依赖模式切换）；
   每条含**地图缩略图 + 地图名（点击进地图详情）、玩家头像 + 玩家名（点击进资料页）、
   获取分数、完成耗时、记录上传时间**；一列内完整展示，**不做横向滚动**；
   超长玩家名/地图名省略号 + hover 全称。
   说明：KZTimer 只对每张地图/关卡前 1000 名计分，超出范围仍会记录但 `points=0`，
   前端将其展示为「未计分」（hover 有规则说明）。

设计约束（必须保留）：

- 浅色简约主题，白底 + 单一墨绿主色（Naive UI 主题覆写 `#2F6F4F`）；以细边框 + 克制的层级阴影
  建立层次，不使用发光/霓虹；关键操作（按钮）必须有明确的填充或描边，不得与普通文本同弱。
- 克制干净，不做科技感/竞技风。
- 响应式布局（基础支持），信息层级清晰。
- 模式：KZT / SKZ / VNL；字体：Inter。
- 底部栏保持紧凑（内边距与间距已压薄，不得随意加高）。
- 顶部导航栏：链接永不断行，窄屏横向滚动或换两行，
  不得出现文字重叠；新增导航项时同步验证 1100 / 860 / 600 三档断点。

视觉系统（2026-10-18 强化后，新增页面必须复用）：

- 设计 token 唯一出口 `frontend/src/styles/global.css`（颜色/圆角/阴影/难度色/字体），
  Naive UI 覆写唯一出口 `frontend/src/app/theme.ts`；页面不得自造颜色与阴影值。
- 页面骨架：`.page-head`（可配 `.page-head-row` + `.page-head-actions`，标题区放计数徽章）
  + `.panel`/`.panel-head`/`.panel-body`（表格与榜单统一容器，常驻微阴影、表头浅绿渐变）；
  统计卡等可点击卡片加 `.hover-lift`。
- 交互层次：导航当前项为**墨绿实心**；主操作按钮（如「加入游戏」）用 `type="primary"`，
  次操作用默认按钮（白底 + 强描边 + 微阴影），hover 浮起；玩家名等实体用墨绿文字链接；
  表格表头浅绿底 + 墨绿字。
- 公共组件：`PlayerCell`（头像 + 名称 + SteamID64）、`RankBadge`（前 3 名金银铜名次徽章）；
  表格列渲染用 `h()` 创建的类不会被 scoped CSS 命中，必须用组件或 `:deep()`。
- 地图/服务器缩略图必须提供缺图占位（显示地图名/图标）；Hero 使用极浅墨绿晕染，
  不做荧光/科技纹理。

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
| 数据接入 | `reqwest` + Tokio（`src/sources/` + `sync_jobs`） | 对接 GlobalAPI 生态公开 API，见 §9；按需回源，**禁止一次性全量镜像** |
| 计分 | 本站自研（标准 GOKZ 公式，`modules/scoring/`） | 独立计分：只读本站 `records` 表，不依赖外部分数，见 §9.4 |

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
├── sources/           # 外部数据源客户端（gokz_top / kzgo；kztimerglobal 预留），唯一允许发外部 HTTP 处，见 §9
└── modules/<domain>/  # 业务模块，六件套（缺省可省）：
    ├── routes.rs      # 只声明路径映射
    ├── handlers.rs    # 薄层：参数萃取 + 错误映射
    ├── service.rs     # 编排：校验、缓存策略、分页
    ├── repository.rs  # 唯一可碰 DB/外部数据处（占位期返回模拟数据）
    ├── models.rs      # 领域模型（Serialize）
    └── dto.rs         # 查询/提交对象（Deserialize）
```

非 CRUD 模块同样放 `modules/`（见 §9）：`ingest`（外部上报接收）、
`scoring`（计分公式，纯函数，无 IO）、`sync`（按需回源编排 + `sync_jobs` 队列）。

规则：handler 不拼 SQL、不做业务判断；service 不碰传输细节；
repository 之外不得出现 `sqlx::query`；外部 HTTP 只允许出现在 `sources/`，
外部响应 DTO 不得进入 `models.rs`；无数据访问的模块可省略 `repository.rs`。

### 3.4 前端分层（职责固定）

```text
frontend/src/
├── app/          # 初始化（main.ts）、根组件、Naive UI 主题
├── router/       # 路由及标题（权限守卫后续在此加）
├── layouts/      # 页面布局（导航 + 紧凑底部栏）
├── components/   # 跨模块公共组件（有第二个使用者时再下沉，勿提前抽象）
├── features/<domain>/  # 业务模块：页面 + api.ts + types.ts
├── services/http.ts     # 全站唯一 fetch 出口（错误体归一为 ApiError）
├── stores/       # Pinia（app.currentMode 固定 KZT，供地图页使用；全站不再提供模式切换）
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
│   │   ├── components/   # 跨模块公共组件（PlayerCell、RankBadge）
│   │   ├── features/     # home/leaderboard/records/servers/maps/profile/bans/auth
│   │   ├── services/     # http.ts 全站唯一 fetch 出口
│   │   ├── stores/       # Pinia（app.currentMode，固定 KZT）
│   │   ├── composables/  # useApi
│   │   ├── types/        # 公共类型（snake_case）
│   │   └── utils/        # 纯展示函数
│   ├── tests/            #（vitest 规划中）
│   ├── package.json
│   └── vite.config.ts    # /api 代理到 Axum
├── backend/              # Rust + Axum（见 backend/README.md）
│   ├── src/              # main/app/config/state/errors/middleware/infrastructure/sources/modules
│   ├── migrations/       # 0001_init.sql、0002_external_data.sql（sqlx migrate 管理）
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

后端已落地模块（2026-10-17）：`sources/`（外部数据源）、`modules/` 下
`maps` / `servers` / `players` / `stats` / `scoring` / `sync` / `ingest`（上报接收端，详见 §9）；
`docs/data-sources.md`、`scripts/fake-plugin.mjs` 仍为规划项。
运维小工具：`backend/src/bin/ingest-key.rs`（服务器组与上报 key 管理）。

## 5. 数据库设计（`0001_init.sql` 为初始表，`0002_external_data.sql` 为外部数据接入）

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

外部数据接入（`0002_external_data.sql`，2026-10-17 已在测试库应用，见 §9）：

- `maps` 增列：`external_id`、`tier_kzt`/`tier_skz`/`tier_vnl`（按模式 tier，0=未评级）、
  `bonuses`、`workshop_id`、`filesize`、`validated`、`authors TEXT[]`、`synced_at`。
- `players` 增列：`region`、`avatar_synced_at`、`updated_at`。
- `records` 增列：`source`（`plugin`/`gokz_top`/`kztimer`）、`external_id`（UNIQUE）、
  `stage`、`tickrate`、`teleports`、`points_local`（本站口径，可空）、
  `origin_server_id`、`server_name`、`server_group`、`wr_time_ms`、`is_valid`、`updated_at`；
  幂等唯一键 `(player_steam_id, map_id, mode, stage, tickrate, teleports, time_ms, created_at)`。
  （原 `points` 列即上游口径分。）
- `servers` 增列：`external_id`（UNIQUE）、`country`、`city`、`region`、`group_name`、
  `group_custom_id`、`is_public`、`online`、`last_seen_at`。
- `bans` 主键改为自增 `id`（同一玩家可多条封禁），增列 `external_id`（UNIQUE）、
  `ban_type`、`stats`、`expires_at`、`source`。
- 新增 `player_scores(player_steam_id, mode, points, points_local, rank, rating,
  maps_completed, wrs_nub, wrs_pro, records_900_plus, records_800_plus, last_played,
  source, updated_at)`：榜单汇总（上游口径 + 本站口径）。
- 新增 `sync_jobs(kind, key, status, attempts, next_run_at, last_error, ...)`：按需回源队列
  （`kind` = `maps`/`leaderboard`/`recent`/`map`/`player`/`servers`/`bans`）。
- 新增 `server_groups(id, name, custom_id)`、`server_keys(group_id, api_key, label, ...)`：
  服务器组与上报 key（§9.6 的 ingest 接收端已使用：`X-ApiKey` → `server_keys`）。

地图/服务器展示补充（`0004_map_meta.sql`，2026-10-17 已应用）：

- `maps.mappers TEXT[]`：制作人显示名（gokz.top 只给作者 SteamID，名称取自 kzgo 地图元数据；
  实测 963/1044 张有值，其余前端显示"作者未知"）。
- `servers.map_tier`：当前地图难度（同步时写入；**读取时用 `maps` 关联覆盖**，避免同步先后导致 0）。
- `servers.player_names TEXT[]`：实时在线玩家昵称（来自上游 `live_status.players[].name`）。

## 6. API 规范

- 前缀：`/api/v1`，JSON，UTF-8。已落地（2026-10-17）：
  - 健康：`GET /api/v1/health`、`GET /api/v1/health/ready`
  - 榜单：`GET /api/v1/leaderboard?mode=KZT`（本站名次，按本站汇总分；前端本地搜索/筛选/排序）
  - 记录：`GET /api/v1/records`（三模式各最新 20 条）
  - 封禁：`GET /api/v1/bans?query=&page=&per_page=`（分页信封）
  - 地图：`GET /api/v1/maps?query=&tier=&mode=`（返回含 `mappers` / 三模式 tier / `bonuses` /
    当前模式 WR `wr_player`·`wr_time`·`wr_date`）、
    `GET /api/v1/maps/{id}?mode=KZT`（详情含主关前 100 与 `syncing` 标记）
  - 服务器：`GET /api/v1/servers`（仅在线 KZ CS 服；含 `map_tier` / `player_names` / `category` / `source`；
    ip/port 仅供复制按钮，页面不渲染明文）
  - 服务器上报（§9.11）：`POST /api/v1/ingest/servers`（X-ApiKey，插件上报实时人数/地图/玩家，
    与 Steam 发现以 `(ip, port)` 合并，插件优先）
  - 玩家：`GET /api/v1/players/{steam_id64}`（个人资料，首次访问触发后台同步）
  - 记录：`GET /api/v1/records`（含 `steam_id64` / `map_id`，供头像、资料页与地图详情跳转；
    含 `created_at` 上传时间）
  - 统计：`GET /api/v1/stats`（首页计数）
- 上报接收端（§9.6，已落地）：`POST /api/v1/ingest/records`（自建插件批量上报，
  **幂等**，逐条返回 `accepted|duplicate|rejected`，单批上限 200）、
  `POST /api/v2.0/records`（GlobalAPI 兼容层，请求体与官方一致，
  返回 `place/top_place/top_overall_place`）；两者均用 `X-ApiKey`（服务器组 key）鉴权，
  错误体沿用统一信封。
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
- 外部数据源：只允许在 `backend/src/sources/` 使用 HTTP 客户端；必须带可识别 UA、
  超时与有限重试，禁止在 handler/service 直接发外部请求；上游字段映射集中在 sources 层
  （见 §9.2/§9.9）。

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
   - 涉及外部数据源、同步策略、计分规则或接收端契约时，同步更新 §9。
   - 无 PG/Redis 时不得阻塞启动（懒连接），占位 `allow(dead_code)` 写清消除条件。

## 9. 外部数据接入与独立计分（GlobalAPI 生态）

> 2026-10-17 定稿。本节是「数据从哪来、分怎么算、多站如何互不影响」的唯一依据，
> 代码落地前后都以此为准；相关表结构与接口已在 §5 / §6 标注落地状态。

### 9.1 生态机制（必须先理解，否则方案会走偏）

- 游戏服务器侧 = **GOKZ**（SourceMod 计时/模式/判定）+ **GlobalAPI 客户端插件**
  （上报成绩；API key 读 `cfg/sourcemod/globalapi-key.cfg`，请求头 `X-ApiKey`）。
- 官方上报接口固定为 `POST {baseUrl}/records`，请求体
  `{ steam_id, map_id, mode, stage, tickrate, teleports, time }`，
  响应回 `place / top_place / top_overall_place`（GOKZ 用它播报"新纪录"）。
- **`baseUrl` 是编译期常量**（生产 `https://kztimerglobal.com/api/v2.0`、
  预发 staging），GlobalAPI 插件只有 `GlobalAPI_Debug` / `GlobalAPI_Staging` 两个
  convar，**不支持自定义 URL** → 服务器改配置无法把记录推到我们站。
- GOKZ 只在 `GlobalsEnabled(mode)` 时上报，条件含 API key 有效、禁用命令检查、
  settings enforcer、地图与模式检查（即第三方站后台显示的
  "apiKey / plugins / settingsEnforcer / map" 检查项）。
- 网站只有两种角色：**只读型**（读别人的公开聚合 API，如 kzgo.eu 读 kztimerglobal）、
  **导入型**（从 GlobalAPI 体系导入，如 gokz.top 的 `globalapiRecordsSync`）。
- **红线**：没有任何站点能拿到"未接入 GlobalAPI 上报"的服务器的成绩。
  我们要拿到那部分数据，只能靠自建上报插件（§9.7）。

### 9.2 数据源与优先级（全部为匿名可读公开接口）

| 用途 | 主数据源 | 说明 |
|---|---|---|
| 最新成绩（实时） | KZTimer `/records/top/recent` | 官方公开广播式接口，60s 轮询增量，`source='kztimer'` |
| 地图 / 个人成绩（按需） | gokz.top（缓存）+ KZTimer | 地图详情、个人记录，优先本地缓存 |
| 地图元信息（tier/bonus/作者） | gokz.top（启动全量 + 每日刷新，落库缓存） | KZTimer `/maps` 只有单一 `difficulty`，无 bonus/作者 |
| 排行榜聚合 | gokz.top（每日刷新，落库缓存） | KZTimer `player_ranks` 的 KZT 恒 500，不可用 |
| 封禁 | gokz.top（每小时刷新，落库缓存） | KZTimer `/bans` 亦可用 |
| 服务器（实时） | ① 本站插件上报（优先）② Steam Web API | 见 §9.11；只展示 KZ 模式 CS 服 |
| 地图作者显示名 | `kzgo.eu/api/*` | 公开只读；字段可能含 null |

> 原则（2026-10-18 起）：成绩实时链路用官方 KZTimer；gokz.top 只作为**可落库缓存的辅助源**
> （地图元信息/排行榜/封禁），任一时刻 gokz.top 不可用不影响已缓存数据展示，也不阻塞实时成绩；
> 服务器改为插件 + Steam 公开 API，不依赖 gokz.top。

关键接口（均已实测）：

- 玩家：`GET /v1/leaderboards/players/{steamid64}`、`GET /v1/records?steamid64=&offset=&limit=`
- 榜单：`GET /v1/leaderboards/players?offset=&limit=`（返回 `count`，可分页）
- 地图：`GET /v1/maps`、`GET /v1/records?map_name=&stage=&limit=`
- 增量：`GET /v1/records/recent?created_since=`
- 其他：`GET /v1/servers`、`GET /v1/bans`、`GET /v1/records/pb?map_name=&identifier=`

注意：`kztimerglobal` 的 `player_ranks`（现成积分总榜）**服务端 500、不可依赖**
→ 分数必须自建（§9.4）。

### 9.3 三条数据通路（统一写入本站 `records` 表）

| 通路 | 触发方式 | 说明 |
|---|---|---|
| 导入 | 定时 + 按需回源 | 从 §9.2 数据源导入历史与外部成绩，`source='gokz_top'|'kztimer'` |
| 原生上报 | `cngokzapi` 插件（后续开发） | `POST /api/v1/ingest/records`，字段最全，见 §9.6 |
| 兼容上报 | 任意 GlobalAPI 兼容客户端（远期） | `POST /api/v2.0/records`，字段与官方一致 |

三条通路的记录**幂等去重**（§9.6）；同一成绩以**插件上报优先**（字段更全），
导入行不重复计分。

### 9.4 独立计分

- 计分完全由本站完成：**只读本站 `records` 表 + 本站计分引擎**；
  **绝不依据外部返回的分数调整自己的分数**，也**不向任何外部站点写分**。
- 规则采用**标准 GOKZ 计分公式**（按地图 tier + 名次 + TP/PRO 口径），
  使"导入的历史成绩"与"将来插件上报的新成绩"处在同一把尺子上。
- 上游 `points` 有两个用途：① 榜单与档案的权威展示值（见下）；② **单测 oracle**——
  拉上游真实成绩，断言本站公式算出的分与之一致。
- 双分存储：`records.points`（单条成绩分，上游口径）、`records.points_local`（本站算，可空）；
  `player_scores.points`（上游全球积分）、`player_scores.points_local`（本站已同步成绩合计）。
- **榜单口径（2026-10-17 实测定案）**：榜单按 `player_scores.points`（上游积分）排序并计算本站名次。
  原因：本站 `points_local` 只有在某玩家成绩被**完整**同步后才与上游等价，而按需回源只拉
  前 1000 条，合计必然偏小；把它当作权威会得出错误排名。个人资料页同时展示两个数字并标注
  "本站已同步合计（可能不完整）"。待插件直报或全量镜像后，再切回本地分优先。
- 本地汇总口径：按 `(地图, 关卡, 是否用传送)` 分组取最佳再求和——TP / PRO 是两张独立榜单、
  各自给分，合并分组会少算（见 `0003_scoring_fix.sql`）。

### 9.5 同步策略（按需回源，禁止一次性全量镜像）—— 已实现

- **玩家维度可懒加载**：用户查某玩家、或收到该玩家上报 → 后台拉其档案与 PB → 落库计分。
- **地图维度必须整图同步**：计分依赖名次，首次需要某 `(地图, 模式, 关卡)` 时，
  拉该图**全部得分记录**（实测单图单模式主关约 2800 条 / 1.2MB / 16 秒），
  之后本地即可实时排名；缓存 TTL 默认 7 天并做增量。
- **榜单维度按页**：用户翻到第 N 页才拉上游第 N 页；另用上游 WR 榜
  （每模式约 100 条）作种子，避免冷启动榜单空白。
- **增量**：定时按 `created_since` 游标拉 `recent`，量很小。
- 所有回源经 `sync_jobs` 队列：同一 key 去重、指数退避、外部总并发上限
  （默认单源 ≤ 2），失败不阻塞请求（先返回骨架/缓存，同步完成后再刷新）。

### 9.6 接收端契约（已实现，2026-10-17）

```http
# 原生：自建 cngokzapi 插件使用（推荐，字段最全）
POST /api/v1/ingest/records            X-ApiKey: <服务器组 key>
[ { "record_uuid": "…", "steam_id64": "…", "player_name": "…",
    "map_name": "…", "mode": "KZT", "stage": 0, "tickrate": 128,
    "teleports": 0, "time_ms": 169422, "server_id": 1394, "created_on": "…" } ]
→ { "results": [ { "status": "accepted|duplicate|rejected", "place": 1, "points": 1000 } ] }

# 兼容层：任意 GlobalAPI 兼容客户端（字段与官方一致）
POST /api/v2.0/records                 X-ApiKey: <key>
{ "steam_id": "STEAM_1:0:…", "map_id": 200, "mode": 200,
  "stage": 0, "tickrate": 128, "teleports": 0, "time": 169.422 }
→ { "place": 1, "top_place": 1, "top_overall_place": 1 }
```

- 鉴权：`X-ApiKey`，key 由本站按**服务器组**签发（`server_groups` / `server_keys`）。
- 幂等键：`(steam_id64, map, mode, stage, tickrate, tp 口径, time_ms, created_on)`
  或客户端 `record_uuid`；重复上报返回 `duplicate`，**不重复计分**。
- 错误体沿用 §6 统一信封；外部上报接口限流并记录来源服务器。
- **实现说明（2026-10-17 已落地）**：
  - 代码位于 `backend/src/modules/ingest/`；鉴权查 `server_keys`（`revoked_at IS NULL`）→ 服务器组。
  - 原生接口单批上限 200 条；**单条失败只让该条 `rejected`**，只有数据库故障才整批 500
    （客户端应重试）。
  - 兼容层官方客户端没有幂等键：用自然键 + **10 分钟时间窗**判定重试；
    `top_place` 与 `place` 同为"含传送口径"名次，`top_overall_place` 为不分传送口径的名次。
  - 受理后按本站口径（`(地图, 关卡, 传送口径)` 取最佳求和）重算该玩家该模式的
    `player_scores.points_local`；**单条成绩分（GOKZ 公式）仍未落地**，
    入库 `points = 0`、响应 `points = null`（见 §9.4）。
  - key 管理：`cargo run --bin ingest-key -- create-group|issue-key|list|revoke|delete-record`。
  - 未做（后续）：请求限流/配额、按服务器组的审计统计、replay / jumpstats 接收。

### 9.7 插件侧约定（自建 `cngokzapi` 插件）

- 定位：**瘦上报器**，不复制 GOKZ 功能；数据从 GOKZ 公开 forward 取
  （`GOKZ_OnTimerEnd_Post(client, course, time, teleportsUsed)` 等，
  头文件见 GOKZ 仓库 `include/gokz/*.inc`）。另可用 `GOKZ_OnTimerStart_Post`、
  `GOKZ_OnRunInvalidated`、`GOKZ_OnJumpValidated` 等。
- 命名：convar 前缀 `cngokz_`、控制台命令 `sm_cngokz_*`；key 文件独立为
  `cfg/sourcemod/cngokzapi/apikey.cfg`。
- **异步 HTTP + 本地队列 + 指数退避**：本站不可用时记录不丢、且**不阻塞游戏线程**。
- 多站分发时**每站一个独立 worker**（独立 key、超时、重试），一站故障不影响其他站
  —— 这是"上传到多个网站互不影响"的实现方式。
- 默认静默（不刷聊天）；选项菜单注册到自己的分类；`Action` 型 forward 永远返回
  `Plugin_Continue`。

### 9.8 共存冲突规避清单（GOKZ + gokz-top-plugins + cngokzapi）

| # | 冲突点 | 规避做法 |
|---|---|---|
| 1 | 命令/convar 重名 | gokz-top 已占 `sm_pb`/`sm_gpb`/`sm_tier`/`gokz_top_*`；我们统一 `cngokz_*`、`sm_cngokz_*`，聊天命令避开 `!pb !tier !wr` |
| 2 | 聊天刷屏 | 记录结束时两边都打印 → 我们的插件默认静默，只做上报 |
| 3 | 选项菜单槽位 | 经 `GOKZ_OnOptionsMenuCreated/Ready` 注册到**自己的分类**，不复用 Jumpstats 等 |
| 4 | forward 返回值 | `Action` 型一律 `Plugin_Continue`，**绝不 `Plugin_Handled`**（会掐断 GOKZ 与其他插件） |
| 5 | HTTP 并发 | 各插件自发出站请求，需各自限速/超时，避免打满出站连接 |
| 6 | 同一成绩被两条通路写入 | 接收端幂等去重（§9.6），否则双计分 |
| 7 | API key 文件 | GlobalAPI 用 `globalapi-key.cfg`、gokz-top 用 `cfg/sourcemod/gokz-top/apikey.cfg`、我们用 `cfg/sourcemod/cngokzapi/apikey.cfg`，互不共用/覆盖 |
| 8 | GOKZ 内部状态 | 只用 GOKZ native 读数据，不自行修改计时/存档 |

### 9.9 合规与礼貌

- 请求带可识别 UA（如 `cngokzapi/0.1 (+联系方式)`）；单源并发 ≤ 2，遇 429/5xx 退避。
- 页面与文档标注数据来源与最后更新时间。
- 只使用公开只读接口；`/v1/admin/*`、需要登录态的接口一律不碰。

### 9.10 待确认项

1. GlobalAPI 客户端未来是否支持自定义 URL（当前不支持）→ 决定"兼容上报"通路的可行性。
2. 是否需要"仅本站服务器成绩"与"全球成绩"两套榜单可切换（`source` 字段已预留）。
3. 上游封禁（`/v1/bans`）是否跟随清榜（策略问题，不阻塞实现）。

### 9.11 服务器来源（2026-10-18 落地）

服务器列表有两个来源，写入同一张 `servers` 表，以 `(ip, port)` 唯一键合并：

| 来源 | 获取方式 | 优先级 | 字段完整度 |
|---|---|---|---|
| 本站插件 | `POST /api/v1/ingest/servers`（`X-ApiKey`，见 §9.6） | 高（10 分钟内不被覆盖） | 最全：名称/地图/人数/在线玩家昵称 |
| Steam Web API | `IGameServersService/GetServerList`（需 `STEAM_API_KEY`） | 低 | 名称/地图/人数，无玩家昵称 |

- Steam 侧按 `appid=730/4465480` + `gametype` 含 `gokz`/`cs2kz`/`kz` 过滤，只保留 KZ 模式 CS 服。
- 同步策略：每 5 分钟拉一次 Steam；开始前先把 `source='steam'` 的置离线，本轮重新发现的写回在线。
- 读取时插件优先：`upsert_steam_server` 在冲突时不覆盖 10 分钟内上报过的 `source='plugin'` 行
  （`upsert_plugin_server` 始终覆盖）。
- `GET /api/v1/servers` 只返回 `online AND is_public` 且（插件服 或 `game_mode` 带 kz）的行，响应含 `source`。
- 运维：`ingest-key delete-server <ip> <port>` 清理误报。

## 10. 当前状态与下一步

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
- 当前：62 测试库已接通（2026-10-16，`develop`）：`.env` + `dotenvy` 落地，
  `GET /api/v1/health/ready` 返回 PG/Redis 双 true，`0001_init` 迁移已在 62 库应用；
  数据接口仍走模拟层，下一步换 PG 回源 + Redis 缓存。
- 当前：外部数据接入方案定稿（2026-10-17，`develop`，**仅文档**）：数据源优先级、
  按需同步策略、独立计分与双分存储、两套接收端契约、插件约定与三插件共存规避清单
  写入 §9；**对应代码尚未开始**。
- 当前：真实数据全面替代模拟数据（2026-10-17，`develop`）：
  - 后端：`0002_external_data.sql` 已应用；新增 `sources/`（GOKZ.TOP 客户端：UA/限速/重试）、
    `modules/sync`（`sync_jobs` 队列 + 后台 worker：启动种子、定时增量、按需回源、
    并发上限、指数退避、重启接管 running 任务）、`modules/scoring`（最佳成绩汇总口径）
    与 `maps`/`servers`/`players`/`stats` 四个读模块；`leaderboard`/`records`/`bans`
    改读本站 PG，**`modules/mock.rs` 已删除**。
  - 数据：实测已导入约 1000+ 张地图、1100+ 玩家、8 万+ 成绩、81 台在线服务器、500 条封禁；
    `GET /api/v1/health/ready` 与各接口均返回真实数据。
  - 前端：`home`/`maps`（含 `/maps/:id` 详情）/`servers`/`profile`（含 `/profile/:steam_id?`）
    四页接真实接口；新增公共组件 `PlayerCell.vue`；导航新增「个人资料」，页脚标注数据来源；
    移除全部占位文案。
  - 验证：`cargo test`（16 passed）、`cargo clippy --all-targets -- -D warnings`、
    `npm run build`（含 `vue-tsc`）全过；直连 18091 逐个接口核对（榜单/记录/封禁/地图/
    地图详情/服务器/档案/统计）。
- 当前：上报接收端落地（2026-10-17，`develop`）：`modules/ingest` 提供
  `POST /api/v1/ingest/records`（批量、逐条结果、单批上限 200）与
  `POST /api/v2.0/records`（GlobalAPI 兼容）；`X-ApiKey` → `server_keys` 鉴权，
  原生按 `record_uuid`、兼容层按自然键 + 10 分钟窗口幂等；受理后重算本地汇总分；
  新增 `src/lib.rs`（供 bin 与测试使用）与运维工具 `src/bin/ingest-key.rs`。
  实测：无/坏 key 401、合法条 `accepted(place=1)`、非法条 `rejected(带原因)`、
  重复 `duplicate`、兼容层 `201 {place,top_place,top_overall_place}` 且二次提交不产生重复行；
  `cargo test` 21 passed。
- 当前：首页公告 / 地图卡片 / 服务器双视图 / 记录头像（2026-10-17，`develop`）：
  地图页改卡片网格（图片/名称/制作人/难度/Bonus，每页 24 张）；服务器页支持卡片与列状两种视图
  （默认列状，均显示地图图片与在线玩家标签）；最新记录加玩家头像并固定列宽防换行；
  首页恢复「最新公告」。后端补 `maps.mappers`、`servers.map_tier`/`player_names`、
  `/records` 的 `steam_id64`，并修正同步队列优先级（全局任务优先于玩家补全）。
  实测制作人 963/1044、在线服务器 89 台其中 42 台有在线玩家昵称；`cargo test` 23 passed。
- 当前：全站视觉优化（2026-10-17，`develop`）：在保持 §2 设计约束的前提下提升信息层次——
  `global.css` 落地面板/工具条/徽章/淡入等统一视觉系统，导航与首页 Hero 重做，
  榜单/封禁表格统一进面板，地图卡片与服务器双视图加 hover 与缺图占位，
  地图详情新增英雄区，个人资料新增头像英雄卡；新增公共组件 `RankBadge`，
  修复 DataTable 列渲染类不命中 scoped CSS、`PlayerCell` SteamID64 换行、
  600px 统计卡挤出 4 列等问题；`npm run build` 通过，1440/1100/860/600 四档截图核对，无 API/DB 变化。
- 当前：地图列表 4 列 + 当前模式 WR（2026-10-18，`develop`）：地图卡片改为宽屏默认 4 列
  （1100 / 860 / 520 降 3 / 2 / 1 列），并在卡片显示当前模式主关 WR：记录人 + 耗时 + 日期。
  后端 `GET /api/v1/maps` 用 `LEFT JOIN LATERAL`（复用 `idx_records_map_place`，按 `time_ms` 取一条）
  追加 `wr_player`/`wr_time`/`wr_date`，地图详情复用主关榜第一名填充；`cargo test` 23 passed、
  `clippy`/`fmt` 与 `npm run build` 均过。
- 当前：服务器页默认卡片 + 社区分类（2026-10-18，`develop`）：视图默认改为卡片排列；
  新增社区分类筛选栏（计数倒序、可横向滚动），卡片/列状均显示社区标签。
  后端新增 `modules/servers/category.rs`（纯函数 + 单测）：优先 `group_name` 且去掉末尾通用词
  （`AXE GOKZ`→`AXE`），缺失时从名称推导（`【HIME】…`→`HIME`、`House of Climb  EU …`→`House of Climb`、
  `KZMANIA GOKZ 2`→`KZMANIA`），`GET /api/v1/servers` 新增 `category`；实测 89 台归为 20 类
  （AXE 22 / House of Climb 16 / CNGOKZ 15 / HIME 3 …）；`cargo test` 26 passed、`clippy` 与前端构建均过。
- 当前：加入游戏按钮 + 视觉强化（2026-10-18，`develop`）：服务器卡片/列状均新增
  「加入游戏」（`steam://rungameid/4465480//+connect ip:port`，地址不写入 DOM），与「复制 IP」并列；
  全站强化视觉层次（§2 视觉系统已更新）：导航当前项墨绿实心、按钮主/次分明且 hover 浮起、
  表头浅绿底、面板/卡片常驻微阴影、卡片头浅绿渐变、玩家名改墨绿链接、页面顶部极浅墨绿晕染；
  `npm run build` 通过，1440 截图核对各页。
- 当前：KZTimer 实时成绩 + 双来源服务器（2026-10-18，`develop`）：
  最新记录改从 KZTimer `/records/top/recent` 拉取（60s 轮询，`source='kztimer'`）；
  新增 `sources/kztimer.rs`、`sources/steam.rs`；服务器改为 Steam Web API（`STEAM_API_KEY`）+
  插件上报（`POST /api/v1/ingest/servers`），`servers` 增 `source/appid/game_mode/steam_id/reported_at`
  并以 `(ip, port)` 合并、插件优先、只显示 KZ 服；`0005_server_sources.sql` 已应用。
  实测 Steam 侧 529 台 KZ 服、插件上报端点 200/401 与优先覆盖均正确；地图元信息/排行榜/封禁仍在
  gokz.top 落库缓存（§9.2），后续可继续迁移到 KZTimer/本地计分。
- 当前：移除导航模式切换 + 三模式排行榜（2026-10-18，`develop`）：导航栏删除 KZT/SKZ/VNL
  切换按钮，`stores/app.ts` 简化为固定 `currentMode='KZT'`（仅地图页使用）；排行榜改为一次拉取
  三模式、**KZT / SKZ / VNL 三列同页并排**（每列前 50 名），行内两行式展示名次/头像/玩家/总分/
  图数/记录数/均分/最近游玩，搜索与筛选排序同时作用于三列；`npm run build` 通过，1440 截图核对。
- 下一步（按序）：
  1. ~~旧阶段（vanilla 原型 + 单文件后端）~~ ✅ 已归档/替换
  2. ~~数据接入落地（§9）：`sources/` + `sync_jobs` + `scoring` + `0002/0003` + `ingest`；
     真实数据点亮全部页面~~ ✅ 已完成
  3. 单条成绩计分：本地实现标准 GOKZ 公式（用上游 `points` 做 oracle 单测），
     落地后 `records.points_local` 与榜单可切回"本地分优先"（§9.4）
  4. 数据完善：玩家头像懒加载（Steam 社区 XML）、档案/榜单过期重同步、Redis 缓存
  5. Steam OpenID 登录 + 管理接口（删记录、封禁）+ 上报限流与审计
  6. `cngokzapi` 服务器插件（瘦上报器，见 §9.7）—— 对接已就绪的 `/api/v1/ingest/records`
