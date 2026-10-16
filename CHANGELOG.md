# 开发日志（CHANGELOG）

> 记录已经发生的事情：每次开发修改了什么、为什么修改、解决了什么问题、
> 修改了哪些文件、数据库和 API 有什么变化，以及什么时候完成。

格式约定：

- 按日期倒序追加，标题为 `## YYYY-MM-DD 主题`。
- 每条写清：背景 / 变更内容 / 涉及文件 / 数据库与 API 变化 / 测试与结果。

---

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
