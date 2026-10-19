# 项目规范与技术设计（PROJECT）

> 记录项目应该如何运作：整体目标、功能需求、技术架构、目录结构、
> 数据库设计、API 规范、编码规范和开发流程。
> AI 开发前必须先读本文件与 CHANGELOG.md，开发后必须同步更新两者。

## 1. 项目整体目标

为中国 GOKZ 社区（CNGOKZ）提供后端 API + 官方网站。正式技术栈：

- 前端：Vue 3 + TypeScript + Vite，组件库 Naive UI（主题覆写为朱砂红主色）；
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
2. 排行榜：**KZT / SKZ / VNL 三模式同页并排**（全量导入：KZT≈10.3 万 / SKZ≈9.4 千 /
   VNL≈3.4 千，独立计分；每列每页 50 人、独立分页），
   每行含名次徽章、头像、名称（链接到资料页）、总分、完成地图、记录数、平均分、最近游玩；
   支持名称模糊 / SteamID64 子串搜索、活跃度筛选（近 7 / 30 天、超 30 天未玩）、多字段排序
   （总积分 / 完成地图 / 平均分 / Rating），**搜索、筛选、排序、分页全部由服务端完成**；
   **导航栏不再提供模式切换**。榜单每 10 分钟刷新前 1000 名，每 6 小时全量回扫一次。
3. 地图列表：浏览全部地图，支持难度（按当前模式 Tier）筛选与名称搜索；
   **卡片网格排列**（仿 gokz.top，宽屏默认 4 列，窄屏降为 3 / 2 / 1 列）：地图图片、
   地图名称、制作人、难度徽章、是否有奖励关（Bonus ×N / 无 Bonus），
   并显示当前模式主关 WR（记录人 + 耗时 + 日期）；每页 24 张分页。图片走 `utils/mapImage.ts`
   统一出口（当前用 KZGlobalTeam 公开图库，后续可换自建图床）。
4. 地图详情：**大图** + 地图名 / 制作人 / KZT·SKZ·VNL 三模式等级 / 文件大小 / Bonus / 创意工坊；
   顶部「三模式最快完成」同时展示 PRO（无存读点）与 TP（有存读点）；
   页内 KZT/SKZ/VNL 切换，每个模式展示两张主关前 100 榜：PRO 与 TP；
   两榜**双列并排**（≤1100px 堆叠），紧凑列宽（紧凑头像 + 无 SteamID64 + `YY-MM-DD HH:mm`）
   确保不出现横向滚动；每条含头像、名称（链接资料页）、完成时间、分数、上传时间，TP 榜额外显示存读点次数。
5. 服务器列表：全部在线 **CS:GO** 服务器（**排除 CS2，appid 730**）详细信息，提供**两种视图切换**——卡片排列（grid，**默认**）与
   列状排列（list）；两种视图都显示当前地图图片、地图名、难度与区域、人数、社区标签、
   在线玩家昵称标签。支持**按社区分类筛选**（后端 `servers/category.rs` 归一化：优先 `group_name`，
   缺失时从服务器名推导，如 CNGOKZ / AXE / HIME；分类栏计数倒序）。
   地址一律不渲染明文：提供「复制 IP」与「加入游戏」两个按钮，后者点击时用
   `steam://rungameid/<CS_APP_ID>//+connect ip:port` 拉起 Steam 客户端
   （`CS_APP_ID` 常量默认 `4465480`，集中在前端 `ServersPage.vue`，便于后续更换）。
6. 个人资料：玩家统计、完成地图、个人记录；最近完成地图与「已同步成绩」计数均**只统计有分记录
   （`points > 0`）**，不展示未破个人记录/未计分的完成。**最近完成地图按 KZT/SKZ/VNL 三列分组**，
   每条含地图缩略图 + 地图名（点击进地图详情）、关卡、完成时间、分数、日期；每模式最多 20 条。
   **地图进度**：页内 KZT/SKZ/VNL 切换 + 「已完成 / 未完成」两个标签页，每页 24 张地图卡片
   （缩略图、地图名、当前模式 Tier、已完成附最佳耗时与分数），支持地图名搜索与分页；
   已完成/未完成以该模式主关（`stage=0`）有效成绩是否存在为准。
   页面视觉（2026-10-19 重设计）：身份卡（真实头像 + 国旗角标 + 可复制 SteamID64 + 国家名 / 区域），
   下方 KZT/SKZ/VNL 三张模式档案卡（各含全球总积分、全球排名、等级与 10 级进度条、完成地图 / WR /
   本站已同步）；顶部玩家搜索框为大号搜索卡（图标 + 主按钮 + 格式提示 + 「高级查询」入口）。
   等级由 Rating 判定，与 GOKZ.TOP 口径一致：New 1.00 / Beginner 2.00 / Amateur 3.00 /
   Casual 4.00 / Regular 5.00 / Skilled 6.00 / Expert 7.00 / Pro 8.00 / Master 9.00 / Legend 10.00，
   Rating < 1 视为「未评级」。国家（ISO 3166 alpha-2，国旗用 flagcdn 图片，缺图只显示文字）与
   真实头像源自 GOKZ.TOP `GET /v1/players/{steamid64}`，随玩家档案同步写入。
7. Steam 登录：当前为模拟，需接 Steam OpenID 真实登录。
8. 封禁名单（新增）：封禁玩家公示，每行含头像、名称、SteamID64、封禁理由、
   封禁时间、封禁服务器、详细数据（地图·模式·异常成绩·Demo 编号）；支持名称模糊 /
   SteamID64 子串搜索，每页 20 人翻页；当前 45 条模拟占位，真实管理数据接入时字段保持兼容。
9. 最新记录：KZT / SKZ / VNL 三列并排，各列最新 20 条（不依赖模式切换）；
   **仅展示有分记录（`points > 0`）**，未打破自己记录/未进入计分范围的完成不显示；
   每条含**地图缩略图 + 地图名（点击进地图详情）、玩家头像 + 玩家名（点击进资料页）、
   获取分数、完成耗时、记录上传时间**；一列内完整展示，**不做横向滚动**；
   超长玩家名/地图名省略号 + hover 全称。
   说明：KZTimer 只对每张地图/关卡前 1000 名计分，超出范围仍会记录但 `points=0`，
   前端将其展示为「未计分」（hover 有规则说明）。
10. 玩家查询：导航「玩家」页，支持输入 SteamID（`STEAM_x:y:z`）、SteamID3（`[U:1:w]` / 纯数字账号 id）、
    SteamID64、Steam 个人主页链接（`/profiles/…` 或 `/id/…`）；`/id/` 自定义名经 Steam Web API
    `ResolveVanityURL` 解析，成功后退转到个人资料页。页面视觉（2026-10-19 重设计）：
    Hero 标题 + 大号搜索卡（实时格式识别条：显示识别出的格式与 SteamID64 预览，无法识别时输入框转警示态）、
    4 个可点击示例 chip（点击即查询）、最近查询（`localStorage` 按 SteamID64 去重、最多 6 条，可清除）、
    格式说明 2×2 卡片（点击即用示例查询）与「如何找到自己的 SteamID」提示卡；≤980px 单列堆叠。

设计约束（必须保留）：

- 浅色简约主题：以**暖调朱砂红（`#C1432B`）为主色**，辅以松绿 `#1F6F6B`、赭石 `#B07D2A`、
  石板蓝 `#4B6472`，并用 KZT/SKZ/VNL 三模式专属色建立区分；以细边框 + 克制的层级阴影建立层次，
  **平涂不上渐变、不使用发光/霓虹**；关键操作（按钮）必须有明确的填充或描边，不得与普通文本同弱。
- 克制干净，不做科技感/竞技风（多色只用于语义区分与点缀，不堆切色块与渐变）。
- 响应式布局（基础支持），信息层级清晰。
- 模式：KZT / SKZ / VNL；字体：Inter。
- 底部栏保持紧凑（内边距与间距已压薄，不得随意加高）。
- 顶部导航栏：链接永不断行，窄屏横向滚动或换两行，
  不得出现文字重叠；新增导航项时同步验证 1100 / 860 / 600 三档断点。

视觉系统（2026-10-19 暖调色彩体系重做后，新增页面必须复用）：

- 设计 token 唯一出口 `frontend/src/styles/global.css`（颜色/圆角/阴影/难度色/字体/模式色/奖牌色），
  Naive UI 覆写唯一出口 `frontend/src/app/theme.ts`；页面不得自造颜色与阴影值，**不得新增渐变**。
- 颜色语义：`--accent` 朱砂红主色（链接/主按钮/表头）、`--accent-2/3/4` 松绿/赭石/石板蓝辅助色、
  `--success/--info/--danger` 语义色；三模式 `--mode-kzt/--mode-skz/--mode-vnl`（朱砂/松绿/赭石），
  配合工具类 `.mode-kzt/.mode-skz/.mode-vnl` 输出 `--mode-color/--mode-soft/--mode-ring`；
  统计卡等用 `.tone-rust/.tone-teal/.tone-ochre/.tone-slate` 输出 `--stat-color/--stat-soft`。
- 难度色为 8 档 `--tier-1`～`--tier-8`（由冷到暖、越高越深，土色系），`.tier-1`～`.tier-8` 直接引用。
- 背景与留白：整体为**低饱和暖灰底**（`--bg-page`），卡片用白色，靠边框与微阴影分层；
  不使用全页渐变/色块背景，避免大面留白与朱砂红互相抢眼。
- 页面骨架：`.page-head`（可配 `.page-head-row` + `.page-head-actions`，标题区放计数徽章）
  + `.panel`/`.panel-head`/`.panel-body`（表格与榜单统一容器，常驻微阴影、表头浅朱砂底）；
  统计卡等可点击卡片加 `.hover-lift`。
- 交互层次：导航当前项为**主色平涂实心**；主操作按钮（如「加入游戏」「查询」）用 `type="primary"`
  （全局覆写为主色平涂 + 主色投影），次操作用默认按钮（白底 + 强描边 + 微阴影），hover 浮起；
  玩家名等实体用主色文字链接；表格表头浅朱砂底 + 朱砂字。
- 公共组件：`PlayerCell`（头像 + 名称 + SteamID64；名称默认链到资料页，`compact` 可隐藏
  SteamID64/缩小头像）、`RankBadge`（前 3 名金银铜名次徽章，用 `--medal-*` 平涂）；
  表格列渲染用 `h()` 创建的类不会被 scoped CSS 命中，必须用组件或 `:deep()`；
  Boolean prop 未传时 Vue 会转为 false，需要默认 true 请用 `withDefaults`。
- 地图/服务器缩略图必须提供缺图占位（显示地图名/图标）；Hero 使用极浅的朱砂单色晕染，
  不做多色「极光」色块与荧光/科技纹理。

已接入真实数据（2026-10-19 核实）：排行榜、地图与地图详情、服务器、最新记录、封禁名单、
玩家资料（积分/排名/等级/头像/国旗/地图进度）均读本站 PG 的真实同步数据。
仍待后端支持的能力：

- Steam OpenID 真实登录（当前入口明确提示未接入，不做假登录）；
- 管理员功能（删记录、封禁玩家）与后台审核；
- 首页「最新公告」为运营文案，暂无上游数据源，仍为静态内容。

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
├── composables/  # useApi（{ data, loading, error, reload } 范式）+ useLiveReload（SSE 自动刷新）
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
│   │   ├── features/     # home/players/leaderboard/records/servers/maps/profile/bans/auth
│   │   ├── services/     # http.ts 全站唯一 fetch 出口
│   │   ├── stores/       # Pinia（app.currentMode，固定 KZT）
│   │   ├── composables/  # useApi / useLiveReload
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
`maps` / `servers` / `players` / `stats` / `scoring` / `sync` / `ingest`（上报接收端，详见 §9）
/ `stream`（SSE 实时推送）；
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
- `players` 增列：`region`、`country`（ISO 3166 alpha-2，来自 GOKZ.TOP `/players/{steamid64}`；
  `0006_player_country.sql`）、`avatar_synced_at`、`updated_at`。
- `0007_player_pb_backfill.sql`（2026-10-19）：无表结构变更，重置 `player` 任务，
  让同步按 PB 重拉以回填「全部已完成地图」。
- `0008_player_profile_backfill.sql`（2026-10-19）：无表结构变更，为所有缺少头像/国籍的玩家
  入队 `player_profile` 轻量任务（每人 1 次 `GET /players/{steamid64}`）。
- `0009_sync_state.sql`（2026-10-19）：新增 `sync_state(key PK, value, updated_at)` 持久化增量游标
  （`kztimer_recent_id` / `gokz_records_updated` / `gokz_bans_updated`）。
- `0010_ban_cascade.sql`（2026-10-19）：把已有封禁玩家的成绩置为 `is_valid=false` 并删除其
  `player_scores`（新封禁由 `run_bans` 实时级联）。
- `0011_leaderboard_full.sql`（2026-10-19）：入队三模式排行榜全量导入任务。
- `0012_sync_last_success.sql`（2026-10-19）：`sync_jobs` 增列 `last_success_at`，供
  `/health/sync` 与 TTL 判断使用（`enqueue` 覆盖 `updated_at` 不影响最后成功时间）。
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
  （`kind` = `maps`/`leaderboard`/`recent`/`map`/`player`/`player_profile`/`servers`/`bans`）。
- 新增 `server_groups(id, name, custom_id)`、`server_keys(group_id, api_key, label, ...)`：
  服务器组与上报 key（§9.6 的 ingest 接收端已使用：`X-ApiKey` → `server_keys`）。

地图/服务器展示补充（`0004_map_meta.sql`，2026-10-17 已应用）：

- `maps.mappers TEXT[]`：制作人显示名（gokz.top 只给作者 SteamID，名称取自 kzgo 地图元数据；
  实测 963/1044 张有值，其余前端显示"作者未知"）。
- `servers.map_tier`：当前地图难度（同步时写入；**读取时用 `maps` 关联覆盖**，避免同步先后导致 0）。
- `servers.player_names TEXT[]`：实时在线玩家昵称（来自上游 `live_status.players[].name`）。

## 6. API 规范

- 前缀：`/api/v1`，JSON，UTF-8。已落地（2026-10-17）：
  - 实时推送：`GET /api/v1/stream?topics=records,map:44,player:<steamid64>,bans,leaderboard,stats`
    （SSE；`topics` 为空订阅全部；事件体 `{ topic, kind, id?, at }`，15s 心跳）
  - 健康：`GET /api/v1/health`、`GET /api/v1/health/ready`、
    `GET /api/v1/health/sync`（各任务族延迟/积压/失败/游标与告警阈值）
  - 榜单：`GET /api/v1/leaderboard?mode=&page=&per_page=&query=&sort=&activity=`
    （分页信封 `{ data, pagination: { page, per_page, total } }`；`sort` = points/maps/avg/rating，
    `activity` = all/7/30/inactive；`rank` 为当前排序下的跨分页名次）
  - 记录：`GET /api/v1/records`（三模式各最新 20 条）
  - 封禁：`GET /api/v1/bans?query=&page=&per_page=`（分页信封）
  - 地图：`GET /api/v1/maps?query=&tier=&mode=`（返回含 `mappers` / 三模式 tier / `bonuses` /
    当前模式 WR `wr_player`·`wr_time`·`wr_date`）、
    `GET /api/v1/maps/{id}`（详情含三模式各自 PRO/TP 前 100、最快成绩与 `syncing` 标记）
  - 服务器：`GET /api/v1/servers`（仅在线 KZ CS 服；含 `map_tier` / `player_names` / `category` / `source`；
    ip/port 仅供复制按钮，页面不渲染明文）
  - 服务器上报（§9.11）：`POST /api/v1/ingest/servers`（X-ApiKey，插件上报实时人数/地图/玩家，
    与 Steam 发现以 `(ip, port)` 合并，插件优先）
  - 玩家解析：`GET /api/v1/players/resolve?q=`（SteamID / SteamID3 / SteamID64 / 个人主页链接 →
    `{ steam_id64 }`；`/id/自定义名` 用 Steam Web API 解析）
  - 玩家：`GET /api/v1/players/{steam_id64}`（个人资料，首次访问触发后台同步；`recent_records`
    按模式各最多 20 条有分成绩，含 `map_id` 供地图图片/详情跳转）
  - 玩家地图进度：`GET /api/v1/players/{steam_id64}/maps?mode=&status=completed|uncompleted&page=&per_page=&q=`
    （已完成/未完成地图分页，返回 `total / completed_total / maps_total` 与 `data[]`；
    已完成项带 `best_time` / `points`；`q` 为地图名模糊搜索）
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
| 最新成绩（实时） | KZTimer `/records/top/recent` | 官方公开广播式接口，**10s 轮询 + record id 游标**，`source='kztimer'` |
| 地图 / 个人成绩（按需） | gokz.top（缓存）+ KZTimer | 地图详情、个人记录，优先本地缓存 |
| 地图元信息（tier/bonus/作者） | gokz.top（启动全量 + 每日刷新，落库缓存） | KZTimer `/maps` 只有单一 `difficulty`，无 bonus/作者 |
| 排行榜聚合 | gokz.top（**前 1000 名每 10 分钟、全量每 6 小时**，落库缓存） | KZTimer `player_ranks` 的 KZT 恒 500，不可用 |
| 封禁 | gokz.top（**每 60s `updated_since` 增量 + 级联失效**，落库缓存） | KZTimer `/bans` 亦可用 |
| 服务器（实时） | ① 本站插件上报（优先）② Steam Web API | 见 §9.11；只展示 KZ 模式 CS:GO 服（排除 CS2 `appid=730`） |
| 地图作者显示名 | `kzgo.eu/api/*` | 公开只读；字段可能含 null |

> 原则（2026-10-18 起）：成绩实时链路用官方 KZTimer；gokz.top 只作为**可落库缓存的辅助源**
> （地图元信息/排行榜/封禁），任一时刻 gokz.top 不可用不影响已缓存数据展示，也不阻塞实时成绩；
> 服务器改为插件 + Steam 公开 API，不依赖 gokz.top。

关键接口（均已实测）：

- 玩家：`GET /v1/leaderboards/players/{steamid64}`、`GET /v1/players/{steamid64}`
  （国家 / 头像 hash / 名称）、`GET /v1/records?steamid64=&offset=&limit=`、
  `GET /v1/records/pb?identifier=&scope=&type=NUB|PRO&is_bonus=&offset=&limit=`（玩家 PB，裸数组）
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

- **玩家维度可懒加载**：用户查某玩家、或收到该玩家上报 → 后台拉其档案、PB 与最近成绩 → 落库计分。
  玩家 PB 按模式分页拉 `GET /records/pb?type=NUB|PRO&is_bonus=false`（每页 1000，上限 20000），
  以保证「全部已完成地图」不被最近成绩截断；旧的 `/records?steamid64=&limit=1000` 仍用于最近成绩。
- **轻量玩家档案**：`player_profile` 任务只请求 `GET /players/{steamid64}`（名称/头像/国家），
  用于批量回填排行榜与地图榜中缺失头像/国旗的玩家；与完整 `player` 任务分开，避免重复拉成绩。
- **地图维度必须整图同步**：计分依赖名次，首次需要某 `(地图, 模式, 关卡)` 时，
  拉该图**全部得分记录**（实测单图单模式主关约 2800 条 / 1.2MB / 16 秒），
  之后本地即可实时排名；缓存 TTL 默认 7 天并做增量。
- **榜单维度按页**：用户翻到第 N 页才拉上游第 N 页；另用上游 WR 榜
  （每模式约 100 条）作种子，避免冷启动榜单空白。
- **增量（2026-10-19 落地，Phase 1-4）**：
  - `recent`（KZTimer）：**10s** 轮询，按 record id 游标只处理新记录，并重算涉及玩家的本站汇总分；
  - `record_updates`（gokz.top `/records?updated_since=`）：**30s** 增量，捕获新成绩、改分与
    **软删除（`is_valid=false`）**，命中已有行则更新状态；封禁/失效成绩会立即从地图榜与个人记录消失；
  - `bans`（gokz.top `/bans?updated_since=`）：**60s** 增量；作弊封禁到达后**级联**把该玩家成绩标为
    无效并删除其榜单汇总（解禁/上游恢复后由 `record_updates` 自动重新计入）；
  - `leaderboard`：由每日改为**每 10 分钟**刷新，前 N 名玩家补全改为 6 小时内不重复入队；
  - 游标持久化在 `sync_state` 表，重启不丢；`upsert_record_state` 兼容 KZTimer/gokz 双来源自然键去重。
- **长期维护（2026-10-19 落地）**：
  - 冷数据 TTL：地图详情 / 玩家 PB 7 天未成功同步则访问时后台重同步（修正硬删除/改分）；
  - 全量榜单结束后清理本期未命中的 `source='gokz_top'` 行，避免掉榜玩家长期残留；
  - `GET /api/v1/health/sync` 输出各任务延迟、pending/running/failed/dead、游标与告警；
  - worker 并发 8（长任务不再饿死 10s/30s 热任务），DB 连接池 16；全量榜单按最后成功时间
    决定是否在重启后重跑，避免每次部署都全量回扫。
- **浏览器推送（SSE，2026-10-19 落地）**：新增 `GET /api/v1/stream`，写入端（ingest / recent /
  record_updates / bans / leaderboard）发布到进程内 `EventBus`；前端 `useLiveReload` 按主题
  （`records` / `map:{id}` / `player:{steamid64}` / `bans` / `leaderboard` / `stats`）400ms 防抖刷新。
  单实例用 tokio broadcast，多实例时可换 Redis Pub/Sub，调用方无需改动。
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
3. ~~上游封禁（`/v1/bans`）是否跟随清榜~~ ✅ 已定稿（2026-10-19）：跟随清榜——封禁级联失效其成绩
   并删除榜单汇总（见 §9.5），解禁后由 `record_updates` 自动恢复。

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
- 当前：地图详情改版（2026年10月19日，`develop`）：`GET /api/v1/maps/{id}` 改为一次返回三模式
  各自的 PRO/TP 前 100（按 `(地图, 关卡, 模式, teleports=0/>0)` 分榜）+ 上传时间/存读点；
  页面重做为「大图 + 三模式等级/制作人/大小/Bonus/工坊 + 三模式最快完成（PRO/TP）+ 页内模式切换
  + PRO/TP 两张全宽排行榜」；TP 表增「存读点」列，上传时间带年份；堆叠布局避免截断。
  实测 `kz_catalyst_gfix` 三模式 PRO/TP 均有数据，`kz_apricity_v3` 本身只有 KZT 数据（非 bug）。
- 当前：过滤无分成绩（2026年10月19日，`develop`）：最新记录与个人资料「最近成绩」只查
  `points > 0`（未破个人记录/未进计分范围的不再显示），个人资料「已同步成绩」计数同口径。
- 当前：个人资料最近成绩分组 + 地图缩略图（2026年10月19日，`develop`）：`PlayerRecordEntry` 增
  `map_id`；`recent_records` 改为按模式查询（每模式 20 条有分成绩）；页面重做为 KZT/SKZ/VNL
  三列，每条含地图缩略图/地图名（点击进 `/maps/:id`）/关卡/时间/分数/日期。
- 当前：导航「玩家」查询（2026年10月19日，`develop`）：新增 `/players` 页与导航项，支持
  SteamID/SteamID3/SteamID64/个人主页链接（含 `/id/` 自定义名经 Steam Web API 解析）；
  后端 `GET /api/v1/players/resolve?q=` 统一返回 `steam_id64`，前端解析后跳转个人资料页。
- 当前：个人资料页重设计（2026年10月19日，`develop`）：身份卡展示真实头像 / 国旗 / 可复制
  SteamID64 / 国家名与区域；下方三张模式档案卡展示全球总积分、全球排名与等级（New 1.00 →
  Legend 10.00，`rating < 1` 未评级，与 GOKZ.TOP 口径一致）；最近成绩区块保持不变。
  后端新增 `players.country`（`0006_player_country.sql`）与 GOKZ.TOP `GET /v1/players/{steamid64}`
  客户端；玩家同步一并写回头像与国籍，并重置 `player` 任务回填历史数据。
- 当前：全站色彩体系重做（2026年10月19日，`develop`）：由「白底 + 单一墨绿」改为暖调多色体系——
  朱砂红主色 + 松绿/赭石/石板蓝辅助色 + KZT/SKZ/VNL 三模式专属色；全局改为**平涂无渐变**，
  `global.css` / `theme.ts` 重写，新增模式与色调工具类、8 档土色系难度色、平涂奖牌色、低饱和暖灰底；
  导航/主按钮/Logo 改为主色平涂，首页四张统计卡分色，榜单/最新记录/个人资料/地图详情按模式着色；
  触及 `DefaultLayout`、`HomePage`、`LeaderboardPage`、`RecordsPage`、`ProfilePage`、`MapsPage`、
  `MapDetailPage`、`ServersPage`、`RankBadge` 与两份 token 文件；无 API/DB 变化。
  （首版为靛蓝/紫罗兰渐变方案，因「AI 感过重」已改为本暖调平涂方案。）
- 当前：个人资料地图进度（2026年10月19日，`develop`）：玩家同步新增 `/records/pb` 分模式分页回源
  （NUB/PRO 主关），修复顶级玩家完成地图被最近 1000 条截断的问题；新增
  `GET /api/v1/players/{steam_id64}/maps`（已完成/未完成、分页、名称搜索）；个人资料「最近成绩」
  更名「最近完成地图」并新增「地图进度」区块（模式切换 + 已完成/未完成标签 + 24 张/页卡片），
  顶部玩家搜索框改为大号搜索卡。
- 当前：真实数据补全（2026年10月19日，`develop`）：新增 `player_profile` 轻量同步任务（仅拉
  `/players/{steamid64}`），迁移 `0008` 为所有缺头像/国籍的玩家入队回填；同步 worker 吞吐
  上调（BATCH 4→8、并发 3→4、TICK 5s→2s，仍受外部 400ms 限速闸门约束）。
  审计结果：头像 163 → **3988/3994**（剩余 6 人 GOKZ.TOP 本身无 `avatar_hash`）、国籍 3024 人；
  排行榜、最新记录、地图详情、封禁名单均改为使用真实头像字段（并修复 `PlayerCell` 因 NAvatar
  default slot 忽略 `src` 的头像 bug）；地图制作人 963/1046（剩余 83 张在 gokz.top / kzgo 均无记录）、
  首页公告为运营文案，均非缺失的接口数据。
  其他真实数据：记录 32.6 万条、在线服务器 619 台、封禁 503 条、地图 1046 张。
- 当前：实时增量同步 Phase 1-4（2026年10月19日，`develop`）：KZTimer recent 60s→**10s** + id 游标；
  新增 `record_updates` 任务（gokz `/records?updated_since=`，**30s**）支持软删除/改分；
  `bans` 改 `updated_since` 增量（**60s**）并**级联失效**封禁玩家成绩；`leaderboard` 24h→**10 分钟**；
  新增 `sync_state` 游标表与 `upsert_record_state`（双来源自然键去重）。
  当前真实延迟：新成绩 10–30s、软删除/封禁 30–60s、排行榜 ≈10 分钟（自建插件接入后可到 <1s）。
- 当前：SSE 页面秒级自动刷新（2026年10月19日，`develop`）：新增 `EventBus`（tokio broadcast）与
  `GET /api/v1/stream`；ingest / recent / record_updates / bans / leaderboard 写入后发布事件，
  前端新增 `useLiveReload`，首页/最新记录/排行榜/地图详情/个人资料/封禁名单订阅对应主题并防抖刷新；
  Playwright 实测地图页、记录页、个人资料页在事件到达后自动重新拉取（无页面跳转），
  单实例内存广播，多实例可换 Redis Pub/Sub。
- 当前：排行榜全量 + 分页（2026年10月19日，`develop`）：上游实际 KZT **103,336** / SKZ 9,415 /
  VNL 3,443，此前仅导入每模式前 200。改为全量导入（`KZT:full` 等任务）+ 每列每页 50 独立分页，
  搜索/活跃度/排序（总积分/完成地图/平均分/Rating）全部下推 SQL；同步策略：前 1000 名每 10 分钟、
  全量每 6 小时，批量 upsert（每页 100 条，jsonb_to_recordset）加速。
  服务器列表排除 CS2（appid 730），只显示 CS:GO；首页在线服务器计数同口径。
- 下一步（按序）：
  1. ~~旧阶段（vanilla 原型 + 单文件后端）~~ ✅ 已归档/替换
  2. ~~数据接入落地（§9）：`sources/` + `sync_jobs` + `scoring` + `0002/0003` + `ingest`；
     真实数据点亮全部页面~~ ✅ 已完成
  3. 单条成绩计分：本地实现标准 GOKZ 公式（用上游 `points` 做 oracle 单测），
     落地后 `records.points_local` 与榜单可切回"本地分优先"（§9.4）
  4. 数据完善：档案/榜单过期重同步（含头像与国籍刷新）、Redis 缓存
  5. Steam OpenID 登录 + 管理接口（删记录、封禁）+ 上报限流与审计
  6. `cngokzapi` 服务器插件（瘦上报器，见 §9.7）—— 对接已就绪的 `/api/v1/ingest/records`
