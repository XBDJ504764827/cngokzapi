# 开发日志（CHANGELOG）

> 记录已经发生的事情：每次开发修改了什么、为什么修改、解决了什么问题、
> 修改了哪些文件、数据库和 API 有什么变化，以及什么时候完成。

格式约定：

- 按日期倒序追加，标题为 `## YYYY-MM-DD 主题`。
- 每条写清：背景 / 变更内容 / 涉及文件 / 数据库与 API 变化 / 测试与结果。

---

## 2026-10-16 接入62测试库：dotenv + 就绪探针 + 迁移落地

- 背景：局域网 192.168.0.62 上有现成测试库（PG 库 cngokzapi / Redis 带密码），把后端真正连上去，结束“无库启动”阶段。
- 变更内容：
  - 新增 `dotenvy`，`main` 启动时加载本地 `.env`（git 忽略）；新增本地 `.env`（62 地址），`.env.example` 默认值同步为 62 测试库。
  - 新增 `GET /api/v1/health/ready`（health 模块 service + handler + route）：返回 `{ status, postgres, redis }`，任一不通即 degraded 不抛错；顺带消除 `state.db/redis` 与 `db::ping` 的 `allow(dead_code)`。
  - 在 62 库执行 `sqlx migrate run`，`0001_init` 已应用（players/maps/records/servers/bans/admins）。
- 涉及文件：
  - 修改：`backend/Cargo.toml`、`backend/src/main.rs`、`backend/src/state.rs`、`backend/src/infrastructure/db.rs`、`backend/src/modules/health/*`、`backend/README.md`、`.env.example`、`PROJECT.md`、`CHANGELOG.md`
  - 新增（未提交）：`.env`（git 忽略）
- 数据库与 API 变化：
  - DB：62 库已建表（迁移 applied）；数据接口仍走模拟层（下次再换 PG 回源）。
  - API：新增 `/api/v1/health/ready`；其余不变。
- 测试与结果（2026-10-16）：
  - `cargo test` 8 passed；`cargo clippy --all-targets -- -D warnings` 通过；`cargo fmt --check` 通过。
  - 运行时（直连 3000，`.env` 生效）：`curl /api/v1/health/ready` 返回 `{"status":"ready","postgres":true,"redis":true}`；榜单仍走模拟（SKZ 首位 JumpGod，120 条）。
  - `git check-ignore .env` 确认凭据文件不会被提交。

## 2026-10-16 正式结构：Vue3+TS+Vite / Axum模块化单体 / PG+Redis

- 背景：旧 vanilla 三件套结构到顶，切换为正式项目结构（前端 Vue 3 + TS + Vite + Naive UI，后端 Rust + Axum 模块化单体，PG + Redis），UI 库选 Naive UI（TS 支持好、主题易覆写）。
- 为什么这样做：
  - 前端按业务模块组织（features/ 下页面 + api.ts + types.ts），新增页面只加目录不碰公共层；全站唯一 fetch 出口收敛错误体。
  - 后端层职责固定（routes 薄 / service 编排 / repository 唯一碰数据），模拟数据下沉到各 repository，上层不动即可替换 PG 回源；PG/Redis 懒连接，无库不阻塞启动。
  - 旧原型移入 `docs/prototype/cngokz-web` 只读参照，不再回写。
- 变更内容：
  - 新增 `frontend/`：8 路由（home/leaderboard/maps/servers/records/profile/bans/login），排行榜/封禁/记录三页经 services 调真实接口完成，其余占位；`stores/app.currentMode` 为全站模式总开关；Vite `/api` 代理到 Axum。
  - 重写 `backend/src`：`main/app/config/state/errors` + `middleware/cors` + `infrastructure/db+cache` + `modules/health+leaderboard+records+bans`（六件套）+ 共享 `page` 分页信封与 `mock` 确定性数据；删除旧 `api/` 与 `router.rs`。
  - 新增 API：`GET /api/v1/leaderboard?mode=`（120/榜，非法 mode 400）、`GET /api/v1/records`（60 条）、`GET /api/v1/bans?query=&page=&per_page=`（分页信封）；`GET /` 改托管 `frontend/dist`（WEB_DIR 默认变更，缺失只告警）。
  - 新增 `backend/migrations/0001_init.sql`（players/maps/records/servers/bans/admins，与 PROJECT §5 一致）；根 `migrations/` 并入 backend。
  - 文档：PROJECT §1/§3/§4/§5/§6/§7/§8 重写（选型表、分层职责、snake_case、分页默认 20），新增 `frontend/README.md`、`backend/README.md`，根 README 重写，`.env.example` 补 DATABASE_URL/REDIS_URL/WEB_DIR。
- 涉及文件：新增约 50 个（frontend 脚手架 + backend 模块 + 迁移 SQL），删除 `web/`（已搬迁）、旧后端 `api/`/`router.rs`。
- 数据库与 API 变化：DB 无运行实例（仅迁移文件落地）；API 见上（新增 3 组，health 不变）。
- 测试与结果（2026-10-16）：
  - 后端：`cargo test` 8 passed（健康/分页/三榜形态/记录/封禁搜索翻页/错误映射）；`cargo clippy --all-targets -- -D warnings` 通过；`cargo fmt --check` 通过。
  - 前端：`npm run build`（vue-tsc + vite）一次通过；`@types/node` 为 vite.config 补装。
  - 联调：`curl` 验证 health/榜单 120 行/非法 mode 400 信封/记录 60/封禁第 3 页 5 行总数 45/搜索命中、`/` 托管 dist；Vite dev 页 + `/api` 代理均通。
  - 修过的问题：sqlx 特性名 `tls-rustls`；`Page` 泛型 Serialize 界；f32/f64；脚手架期 `allow(dead_code)` 均写清消除条件。

## 2026-10-16 顶部导航栏防重叠布局修复

- 背景：加入封禁名单后导航共 6 个链接，空间不足时链接文字被挤换行、与品牌区/右侧操作区视觉重叠。
- 变更内容（纯 `web/styles.css`，HTML/JS 未动）：
  - 链接永不断行：`.nav-link` 加 `white-space: nowrap` + `flex-shrink: 0`，极窄时 `.nav-links` 横向滚动（滚动条隐藏）而非换行重叠。
  - 三区不互挤：`.nav-container` 加 `gap: 16px`，品牌区与操作区 `flex-shrink: 0`，链接区 `min-width: 0` 允许收缩。
  - 腾空间：导航搜索框 220px→170px。
  - 响应式：1100px 以下隐藏副标题、搜索框缩至 150px；860px 以下导航换两行（链接整行置底）；600px 以下隐藏导航搜索框（各页内均有搜索）、Logo 与登录按钮缩小。
- 涉及文件：
  - 修改：`web/styles.css`、`PROJECT.md`、`CHANGELOG.md`
- 数据库与 API 变化：无。
- 测试与结果（2026-10-16）：
  - CSS 大括号平衡检查通过；新规则（nowrap/隐藏滚动条/三档断点/170px/副标题隐藏/换行）齐备；`node --check` 确认 JS 未受影响。
  - 运行时：served `styles.css` 含新规则且 200。

## 2026-10-16 最新记录三列/底部压薄/封禁名单页

- 背景：最新记录只能跟随模式开关看一种模式；全站底部栏太高；需要新增封禁公示页（搜得着、分得页，先模拟占位）。
- 变更内容：
  - 最新记录：删除 `当前模式:KZT` 提示与单表，改为 KZT / SKZ / VNL 三列并排（`records-columns` 网格，1080px 以下堆叠），各列最新 20 条；列内去掉冗余模式列；`recentRecords` 由 9 条静态改为每模式 20 条生成器（每天约两条、日期倒序）；`setMode` 不再刷新记录页（与模式解耦），个人资料页的 `filter(mode).slice(0,5)` 仍兼容。
  - 底部栏：`.footer` 上下内边距 42/20→18/12、`margin-top` 56→32，内容间距与链接行距同步压薄。
  - 封禁名单页：导航新增 `封禁名单`（`pages` 路由同步），表头为玩家（头像+名称+SteamID64）/ 封禁理由 / 封禁时间 / 封禁服务器 / 详细数据（地图·模式·异常成绩·Demo 编号）；45 条模拟数据（3 页，验证翻页）；名称模糊 / SteamID64 子串搜索（搜索回到第 1 页）；每页 20 人，上一页/数字/下一页 + `第 X / Y 页`，单页结果隐藏翻页；计数徽章复用排行榜样式。
- 涉及文件：
  - 修改：`web/index.html`（记录三列、封禁页、导航）、`web/app.js`（记录/封禁生成器与渲染、翻页、搜索联动）、`web/styles.css`（三列、封禁、翻页、底部）、`PROJECT.md`、`CHANGELOG.md`
- 数据库与 API 变化：无（仍为前端模拟数据阶段）。
- 测试与结果（2026-10-16）：
  - `node --check`：通过；新老 ID/CSS 对照：9 个新 ID、8 个新类名齐备，三列 tbody 恰 3 个，无 `recentRecords` 旧引用与 `recordsModeLabel` 残留。
  - 逻辑单测（25 断言全过）：三模式各 20 条且日期倒序、各列渲染 20 行；封禁 45 人字段齐备名称唯一且按封禁时间倒序；第 1 页 20 行 / 第 3 页 5 行 / 越界不跳；名称与 SteamID64 搜索命中；单页隐藏翻页；无匹配空态。
  - `cargo test`：通过（2 passed，后端未动）；运行时（`PORT=18085`）：首页与仓库逐字节一致，served 页含封禁页、三列与翻页标记，`health` 正常。

## 2026-10-16 服务器列式默认/隐藏地址/排行榜计数优化

- 背景：服务器页默认网格视图、加入按钮文字过长折两行、卡片明文展示地址易被爬虫抓取；排行榜计数挤在模式提示行里，位置与文案不合理。
- 变更内容：
  - 服务器默认列式：`serverViewMode` 默认 `grid` 改 `list`，`view-switcher` 的 `active` 换到列表按钮，首屏 `renderServers(serverViewMode)`；网格/列表切换保留。
  - 按钮精简：两处 `加入游戏` 改为 `加入`（复制 IP 按钮不动）。
  - 地址隐藏：删除网格卡片地址行与列表卡片地址（含分隔符），页面不再渲染可见地址；复制 IP / 加入按钮的 `onclick` 参数保留（功能所需，页面上不可见）。
  - 排行榜计数：`mode-hint` 行只保留模式提示，计数移为标题行右侧徽章 `.leaderboard-count-badge`（未筛选显示 `共 120 人`，筛选后显示 `筛出 X 人 / 共 120 人`），窄屏自动堆叠。
- 涉及文件：
  - 修改：`web/index.html`、`web/app.js`、`web/styles.css`（删除无用 `.server-ip` 规则，`.leaderboard-count` 改为徽章）、`PROJECT.md`、`CHANGELOG.md`
- 数据库与 API 变化：无。
- 测试与结果（2026-10-16）：
  - `node --check`：通过；残留检查：`加入游戏` / `server-ip` / 旧 `.leaderboard-count` 均无残留。
  - UI 断言（10 项全过）：默认 list；两种视图均无地址展示元素/标签、按钮为加入、保留复制 IP 与玩家列表；计数徽章两种文案正确。
  - `cargo test`：通过（2 passed，后端未动）；运行时（`PORT=18084`）：`/`、`/app.js`、`/styles.css`、`health` 均 200 且类型正确， served 首页含 badge、list 默认、最近游玩列。

## 2026-10-16 排行榜加厚（头像/最近游玩/100+人/搜索筛选）

- 背景：排行榜每榜仅 3 条、无头像、无最近游玩、搜索还是弹窗模拟，需先用模拟数据把榜单做厚，后续再从外部网站挖掘真实数据填入。
- 为什么这样做：
  - 每模式生成 120 条确定性模拟数据（固定种子伪随机，每次打开一致，方便对照验证），头部保留 ProGamer 等熟人，前 3 名排名色保留。
  - 字段按真实数据先行兼容：`rank / player / steamId64（17 位 76561198 开头）/ avatar（dicebear，与站内头像同源）/ mapsCompleted / totalPoints / avgPoints / records / lastPlayed / lastPlayedDaysAgo`，真实数据接入时只换 `buildMockLeaderboard` 返回值（注释已标明）。
  - 搜索支持名称模糊（不区分大小写）+ SteamID64 子串；活跃度筛（近 7 / 30 天、超 30 天未玩）用 `lastPlayedDaysAgo` 判定；排序支持总分/完成地图/平均分/记录数；筛选条件在模式切换时保留；导航栏搜索框改为联动定位到排行榜（去掉弹窗模拟）。
  - 生成池按模式剔除头部重名，保证每榜 120 个唯一名称（修过 SKZ 重名导致单测失败的问题）。
- 变更内容：
  - `web/index.html`：排行榜表头新增玩家（头像+名称+SteamID64）与最近游玩列，新增搜索框 + 活跃度/排序下拉 + 计数 + 空态 + 模拟数据说明。
  - `web/app.js`：新增 `LEADERBOARD_SIZE=120`、`mulberry32`、`mockAvatar`、`mockSteamId64`、`buildMockLeaderboard`、`getFilteredLeaderboard`、`applyLeaderboardSearch`；重写 `renderLeaderboard`；导航栏搜索联动排行榜。
  - `web/styles.css`：新增 `.leaderboard-toolbar`、`.lb-player`、`.lb-avatar`、`.lb-steam`、`.lb-last-*`、`.leaderboard-empty`、`.leaderboard-note` 及移动端堆叠。
- 涉及文件：
  - 修改：`web/index.html`、`web/app.js`、`web/styles.css`、`PROJECT.md`、`CHANGELOG.md`
- 数据库与 API 变化：无（仍为前端模拟数据阶段；后端 `/api/v1/health` 与静态托管不变）。
- 测试与结果（2026-10-16）：
  - `node --check web/app.js`：通过；ID/CSS 对照脚本：6 个 ID 与 10 个类名齐备，`leaderboardData` 无重复。
  - 逻辑单测（vm 沙箱，31 断言全过）：三榜各 120 人、名称唯一、排名连续、总分降序、SteamID64 格式、头像同源、游玩 0~60 天；名称/SteamID64 搜索命中；三档活跃度筛选分别命中 13 / 53 / 67 人；排序切换有序；无匹配返回空。
  - `cargo test`：通过（2 passed）；`cargo clippy --all-targets -- -D warnings`：通过；`cargo build`：通过。
  - 运行时验证（`PORT=18083`）：`curl /api/v1/health` 正常；`curl /` 与 `/app.js` 与仓库文件逐字节一致。

## 2026-10-16 默认监听全网段（局域网可访问）

- 背景：开发机为 192.168.0.101，服务需供局域网内机器访问，原默认 `HOST=127.0.0.1` 只允许回环，局域网打不开页面。
- 为什么这样做：默认改为 `0.0.0.0` 即监听全部网卡（含 `127.0.0.1` 与 `192.168.0.101`），一台配置同时覆盖本机与局域网；仍可用环境变量收紧（`127.0.0.1` 仅本机 / `192.168.0.101` 仅该网卡）。
- 变更内容：
  - `backend/src/config.rs`：`HOST` 默认值由 `127.0.0.1` 改为 `0.0.0.0`，补注释说明三种取值的含义。
  - `.env.example`：`HOST=0.0.0.0`，补局域网访问注释。
  - `README.md`：修正过时的“骨架/预留”描述，新增“本地运行（局域网）”小节（含 192.168.0.101 访问地址与 HOST/PORT/WEB_DIR 用法）。
- 涉及文件：
  - 修改：`backend/src/config.rs`、`.env.example`、`README.md`、`CHANGELOG.md`、`PROJECT.md`
- 数据库与 API 变化：无（API 路径与返回体不变，只是监听地址变化）。
- 测试与结果（2026-10-16）：
  - `cargo test`：通过（2 passed）；`cargo clippy --all-targets -- -D warnings`：通过；`cargo build`：通过。
  - 运行时验证（`PORT=18082`，默认 HOST）：`ss` 显示 `0.0.0.0:18082`，`curl http://127.0.0.1:18082/api/v1/health` 与 `curl http://192.168.0.101:18082/api/v1/health` 均返回 ok，`curl http://192.168.0.101:18082/` 返回 200 / 18227 字节 / `text/html`。

## 2026-10-16 网站1:1复刻与Rust静态托管（第二步）

- 背景：在 `develop` 上继续第二步：`web/` 做到与 `~/cngokz-web` 一模一样，并由 Rust（Axum）直接托管。
- 为什么这样做：
  - 原型是零构建的 `index.html` + `styles.css` + `app.js`（相对路径引用），直接原样托管是最快且零漂移的“一模一样”方案，不引入前端构建链。
  - 同一个 Axum Router 同时挂 `/api/v1/*`（优先匹配）与静态 fallback，后续第三步把模拟数据换成真实 API 时无需改托管层。
  - `tower-http` 需补 `fs` 特性才能用 `ServeDir`；`WEB_DIR` 环境变量允许部署时覆盖静态目录。
  - `clippy -D warnings` 下无用包装函数会报错，删除 `create_router()` 只保留 `create_router_with_web_dir()`，并补路由构建单测。
- 变更内容：
  - `web/`：`cp` 原样复刻 `index.html` / `styles.css` / `app.js`（sha256 与示例完全一致），删除 `web/PLACEHOLDER.md`（真实文件落地）。
  - 后端：`tower-http` 增加 `fs` 特性；`config.rs` 新增 `WEB_DIR`（默认 `web`）；`router.rs` 改为 `ServeDir::new(web_dir).append_index_html_on_directories(true)` 作 `fallback_service`，`/` 自动返回 `index.html`，删除旧占位 `root()`；`main.rs` 传入 `web_dir` 并打印到日志；`.env.example` 补 `WEB_DIR=web`。
- 涉及文件：
  - 新增：`web/index.html`、`web/styles.css`、`web/app.js`
  - 修改：`backend/Cargo.toml`、`backend/src/router.rs`、`backend/src/config.rs`、`backend/src/main.rs`、`.env.example`、`Cargo.lock`、`CHANGELOG.md`、`PROJECT.md`
  - 删除：`web/PLACEHOLDER.md`
- 数据库与 API 变化：
  - 数据库：无。
  - API：`GET /api/v1/health` 不变；`GET /` 从占位文本变为返回 `web/index.html`；新增静态 `GET /styles.css`（`text/css`）、`GET /app.js`（`text/javascript`）；不存在路径返回 404。
- 测试与结果（2026-10-16）：
  - `sha256sum`：三文件与 `~/cngokz-web` 逐字节一致；`diff -r ~/cngokz-web web/` 仅差 `README.md`（示例独有，不需复刻）。
  - `cargo fmt --all`：通过；`cargo test`：通过（2 passed，含 `health_body_fields` + `default_router_builds`）；`cargo clippy --all-targets -- -D warnings`：通过；`cargo build`：通过。
  - 运行时验证（`PORT=18081`，`web_dir=web`）：`curl /`、`curl /styles.css`、`curl /app.js` 与仓库文件 `diff -q` 均为 IDENTICAL，Content-Type 正确；`curl /api/v1/health` 仍返回 ok；`curl /nonexistent-xyz` 返回 404。

## 2026-10-16 后端骨架与健康检查（第一步）

- 背景：用户已完成首次推送，回到 `develop` 开始第一步：Cargo workspace 初始化 + 后端 Hello World + 健康检查。
- 为什么这样做：
  - 先立住可运行、可测试的 Rust 服务骨架（Axum + Tokio + tracing），后续网站静态托管与 `/api/v1/*` 都挂在同一个 Router 上，避免返工。
  - 健康检查 `GET /api/v1/health` 作为部署与联调的存活探针，字段保持稳定。
  - 遇到 `tower-http 0.6` 中 `TraceLayer::http()` 不存在（应为 `new_for_http()`），按新版 API 修复，保证请求日志可用。
- 变更内容：
  - 新增 workspace：`Cargo.toml`（members = ["backend"]）、`Cargo.lock`（二进制锁定提交）。
  - 新增后端 crate `cngokz-backend`：`backend/Cargo.toml`、`src/main.rs`、`src/config.rs`（HOST/PORT 环境变量）、`src/router.rs`（`/` + `/api/v1/health` + `TraceLayer::new_for_http()`）、`src/api/mod.rs`、`src/api/health.rs`。
  - 新增 `.env.example`（HOST/PORT/RUST_LOG 示例）。
  - 删除占位：`backend/PLACEHOLDER.md`、`backend/src/.gitkeep`（`web/PLACEHOLDER.md` 保留到第二步）。
  - 修正 `.gitignore`：移除 `Cargo.lock` 忽略（视频二进制 workspace 需要提交 lock 文件）。
- 涉及文件：
  - 新增：`Cargo.toml`、`Cargo.lock`、`backend/Cargo.toml`、`backend/src/main.rs`、`backend/src/config.rs`、`backend/src/router.rs`、`backend/src/api/mod.rs`、`backend/src/api/health.rs`、`.env.example`
  - 修改：`.gitignore`、`CHANGELOG.md`、`PROJECT.md`
  - 删除：`backend/PLACEHOLDER.md`、`backend/src/.gitkeep`
- 数据库与 API 变化：
  - 数据库：无。
  - API：新增 `GET /api/v1/health`（无需鉴权，返回 `{ status: "ok", service: "cngokzapi", version: "0.1.0" }`）；`GET /` 暂为占位文本，第二步替换为 web 静态页。
- 测试与结果（2026-10-16）：
  - `cargo fmt --all`：通过。
  - `cargo test`：通过（1 passed，`health_body_fields`）。
  - `cargo clippy --all-targets -- -D warnings`：通过。
  - `cargo build`：通过，生成 `target/debug/cngokz-backend`。
  - 运行时验证（`PORT=18080`）：`curl /api/v1/health` 返回 `{"status":"ok","service":"cngokzapi","version":"0.1.0"}`；`curl /` 返回占位文本；日志可见 `listening on http://127.0.0.1:18080`。

## 2026-10-16 仓库初始化（骨架）

- 背景：开始 `cngokzapi` 开发，先搭仓库骨架，等用户推送到 GitHub 后再做实际开发。
- 变更内容：
  - 本地 `git init -b main`，创建 `main` 与 `develop` 两个分支。
  - 创建项目目录骨架：`backend/`、`web/`、`docs/`、`scripts/`、`migrations/`，目录下暂用占位文件。
  - 新增 `README.md`、`PROJECT.md`、`CHANGELOG.md`、` .gitignore`。
  - 明确技术方向：后端使用 Rust，网站需与 `~/cngokz-web` 示例一模一样（详见 PROJECT.md）。
- 涉及文件：
  - 新增：`README.md`、`PROJECT.md`、`CHANGELOG.md`、`.gitignore`
  - 新增占位：`backend/PLACEHOLDER.md`、`web/PLACEHOLDER.md`、`docs/.gitkeep`、`scripts/.gitkeep`、`migrations/.gitkeep`
- 数据库与 API 变化：无（尚未开始）。
- 测试与结果：
  - `git branch` 确认 `main` / `develop` 均存在。
  - `ls -R` 确认目录骨架完整。
  - 未做功能测试，等待推送后开始网站开发。
