# 开发日志（CHANGELOG）

> 记录已经发生的事情：每次开发修改了什么、为什么修改、解决了什么问题、
> 修改了哪些文件、数据库和 API 有什么变化，以及什么时候完成。

格式约定：

- 按日期倒序追加，标题为 `## YYYY-MM-DD 主题`。
- 每条写清：背景 / 变更内容 / 涉及文件 / 数据库与 API 变化 / 测试与结果。

---

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
