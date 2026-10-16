# 项目规范与技术设计（PROJECT）

> 记录项目应该如何运作：整体目标、功能需求、技术架构、目录结构、
> 数据库设计、API 规范、编码规范和开发流程。
> AI 开发前必须先读本文件与 CHANGELOG.md，开发后必须同步更新两者。

## 1. 项目整体目标

为中国 GOKZ 社区（CNGOKZ）提供后端 API + 官方网站。

- 第一阶段（网站）：用 Rust 承载网站，要求与 `~/cngokz-web` 前端示例**一模一样**
  （视觉、布局、配色、交互、页面结构），先做到像素级复刻，再接后端数据。
- 第二阶段（后端）：提供 RESTful API（玩家、地图、记录、排行、服务器、Steam 登录、管理功能）。
- 第三阶段（联调）：网站从模拟数据切换到真实 API，上线管理面板、搜索、分页、缓存。

## 2. 功能需求（以 ~/cngokz-web 原型为准）

原型位置：`~/cngokz-web/index.html`、`styles.css`、`app.js`，原型 7 页 + 新增 1 页：

1. 首页：社区统计、在线服务器列表、最新记录。
2. 排行榜：全球玩家排名（KZT / SKZ / VNL 各 120 人，独立计分排序），
   每行含头像（模拟图，后续换 Steam 真实头像）、名称、SteamID64、完成地图、
   总分、平均分、记录数、最近游玩时间；支持名称模糊 / SteamID64 子串搜索、
   活跃度筛选（近 7 / 30 天、超 30 天未玩）、多字段排序，计数以标题右侧徽章展示。
   条目字段后续接真实数据时保持兼容，详见 `web/app.js` 中 `buildMockLeaderboard` 注释。
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

- 浅色简约主题，白底 + 单一墨绿主色，细边框代替强阴影/发光。
- 底部栏保持紧凑（内边距与间距已压薄，不得随意加高）。
- 顶部导航栏：链接永不断行（`white-space: nowrap`），窄屏横向滚动或换两行，
  不得出现文字重叠；新增导航项时同步验证 1100 / 860 / 600 三档断点。

设计约束（必须保留）：

- 浅色简约主题，白底 + 单一墨绿主色，细边框代替强阴影/发光。
- 克制干净，不做科技感/竞技风。
- 响应式布局（基础支持），信息层级清晰。
- 模式：KZT / SKZ / VNL；字体：Inter。

待后端支持的能力（原型中为模拟数据）：

- Steam OpenID 登录、实时服务器数据、玩家记录查询、地图库、
  排行榜实时更新、玩家搜索、管理员功能（删记录、封禁玩家）。

## 3. 技术架构

### 3.1 选型

- 后端：Rust + Axum（路由） + Tokio（异步） + SQLx（数据库） + tracing（日志）。
  - 为什么：性能好、内存安全、与后续游戏服务器插件生态契合；
    Axum 与 Tokio 是当前 Rust 主流 Web 组合，SQLx 支持编译期检查 SQL。
- 数据库：PostgreSQL（首选，生产）/ SQLite（本地开发可选）。
  - 为什么：排行榜、前 100 名、分页查询需要强一致的 SQL 与索引；迁移工具成熟。
- 网站第一阶段：Rust（Axum）做静态托管，直接复刻 `~/cngokz-web` 的
  `index.html` + `styles.css` + `app.js`，做到一模一样，不引入前端构建链。
  - 为什么：最快达到“一模一样”，无构建漂移；后续再按需引入模板（Askama）或前端框架。
- 网站第二阶段：保留同一样式，把 `app.js` 中的模拟数据逐页替换为 `fetch(/api/...)` 真实接口。

备选说明：若后续需要复杂交互，可评估 Leptos / Dioxus / React，但必须先经 PROJECT 评审，
且不得改变既有视觉。

### 3.2 运行形态

```text
浏览器
  └─> Rust Web 服务（Axum）
        ├─ GET /            -> 托管 web/ 静态页面（与示例一致）
        ├─ GET /static/*    -> CSS / JS / 字体 / 图片
        └─ GET/POST /api/*  -> 后端 JSON API（第二阶段接真实数据）
```

## 4. 目录结构（目标态，当前为占位）

```text
cngokzapi/
├── backend/              # Rust 后端 crate（Axum + SQLx）
│   ├── src/
│   │   ├── main.rs       # 入口、二进制启动
│   │   ├── router.rs     # 路由装配（/ 与 /api/*）
│   │   ├── api/          # 各资源 handler（players/maps/records/servers/auth/admin）
│   │   ├── db/           # 连接池、查询封装
│   │   └── config.rs     # 配置（环境变量）
│   ├── Cargo.toml
│   └── PLACEHOLDER.md    # 推送前占位，开发时删除
├── web/                  # 网站静态资源（必须与 ~/cngokz-web 一模一样）
│   ├── index.html
│   ├── styles.css
│   ├── app.js
│   └── PLACEHOLDER.md    # 推送前占位，开发时替换为真实文件
├── migrations/           # SQLx / sql 迁移文件（推送后添加）
├── docs/                 # 补充文档（推送后添加）
├── scripts/              # 开发/运维脚本（推送后添加）
├── PROJECT.md
├── CHANGELOG.md
└── Cargo.toml            # workspace（推送后创建，成员含 backend）
```

当前骨架中 `backend/src/`、`docs/`、`scripts/`、`migrations/` 暂用 `.gitkeep` 保留空目录。

## 5. 数据库设计（草案，推送后细化）

表（最小集）：

- `players(steam_id PK, name, avatar_url, points, created_at, banned_until)`
- `maps(id PK, name UNIQUE, tier 1-8, mode KZT/SKZ/VNL, created_at)`
- `records(id PK, player_steam_id FK, map_id FK, mode, time_ms, points, created_at)`
  - 索引：`(map_id, mode, time_ms)` 用于地图前 100；`(player_steam_id, created_at)` 用于个人记录。
- `servers(id PK, name, map, ip, port, player_count, max_players, updated_at)`
- `admins(steam_id PK, role, created_at)`

约定：全部 `created_at/updated_at` 使用 UTC；时间存毫秒整数 `time_ms`；金额/积分用整数。

## 6. API 规范

- 前缀：`/api/v1`，JSON，UTF-8。
- 命名：复数资源小写下划线，如 `GET /api/v1/players/:steam_id`、`GET /api/v1/maps`、
  `GET /api/v1/maps/:id/records?mode=KZT&limit=100`、`GET /api/v1/rankings?scope=total|monthly|weekly`、
  `GET /api/v1/servers`。
- 分页：`?page=1&per_page=50`（默认 50，最大 100），返回 `{ data, pagination: { page, per_page, total } }`。
- 错误：统一 `{ error: { code, message } }`，HTTP 状态码语义正确
  （400 参数错 / 401 未登录 / 403 无权限 / 404 不存在 / 500 服务错），`code` 如 `INVALID_PARAM`。
- 权限：普通接口公开；写操作与管理接口需 Steam 登录 + session/cookie 校验；
  管理员接口再校验 `admins` 表角色。
- 日志：Axum `TraceLayer` 记录请求方法、路径、状态码、耗时；业务错误记 `warn`，系统错误记 `error`。

## 7. 编码规范

- Rust：`cargo fmt` + `cargo clippy -D warnings` 必须通过；模块按资源拆分；
  DB 查询一律经 `db/` 封装，不在 handler 里拼 SQL 字符串。
- 前端复刻阶段：不改视觉（颜色、间距、字体、断点），`styles.css` 原则上原样搬运；
  如需适配 Axum 静态路径，只改引用路径并记入 CHANGELOG。
- 注释：复杂 SQL、排名算法、Steam 流程必须写中文注释说明原因。
- 提交信息：`feat/fix/docs/chore: 简述`，中文，关联页面或 API。

## 8. 开发流程

1. 日常从 `develop` 切功能分支：`feat/web-mirror`、`feat/api-players` 等。
2. 自测：后端 `cargo test`；网站对照 `~/cngokz-web` 逐页比对（首页/排行/地图/详情/服务器/资料/登录）。
3. 合入 `develop` 前更新 `CHANGELOG.md`（按模板写清文件/DB/API/测试）。
4. 稳定后 `develop` -> `main`，打 tag。
5. AI 开发前必须检查：
   - 读 `PROJECT.md` + `CHANGELOG.md` 最新状态。
   - 对照 `~/cngokz-web` 原型确认页面与样式。
   - 确认当前分支（日常在 `develop` 或功能分支，不直接提交 `main`）。
6. AI 开发后必须更新：
   - `CHANGELOG.md` 新增当日条目。
   - 若架构/目录/API/DB 有变，同步改 `PROJECT.md`。
   - 保持占位文件清理（真实文件落地后删除对应 `PLACEHOLDER.md`）。

## 9. 当前状态与下一步

- 当前：第二步完成（2026-10-16，`develop`）：`web/` 三件套与 `~/cngokz-web` 逐字节一致，
  Axum `ServeDir` 托管（`/` 自动返回 `index.html`，`WEB_DIR` 可覆盖），
  `GET /api/v1/health` 与 `cargo test/clippy` 均通过。`web/` 已无占位。
  追加补丁：`HOST` 默认改为 `0.0.0.0`，开发机 192.168.0.101 上局域网可直接访问（`127.0.0.1` 仍可用环境变量切回）。
- 当前：排行榜加厚完成（2026-10-16，`develop`）：每模式 120 人、头像（模拟图）、
  SteamID64、最近游玩时间、名称/SteamID64 搜索 + 活跃度筛选 + 多字段排序，
  导航栏搜索框联动定位到排行榜；`cargo test/clippy` 与静态托管验证均通过。
- 当前：记录三列 + 封禁名单完成（2026-10-16，`develop`）：最新记录三模式并排各 20 条
  （与模式开关解耦）；全站底部栏压薄；新增封禁名单页（45 条模拟、搜索 + 每页 20 人翻页）；
  逻辑单测 25 断言与托管验证均通过。
- 下一步（按序）：
  1. ~~`cargo new` / workspace 初始化，后端 Hello World + 健康检查 `GET /api/v1/health`。~~ ✅ 已完成
  2. ~~`web/` 1:1 复刻 `~/cngokz-web`，Axum 托管静态，对比验证一模一样。~~ ✅ 已完成
  3. 设计迁移 SQL + 上述核心 API，`app.js` 从模拟数据切到真实接口。
