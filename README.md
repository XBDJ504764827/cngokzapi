# cngokzapi

CNGOKZ（中国 GOKZ 社区）后端 + 网站项目。

> 当前阶段：`develop` 上已完成后端骨架 + 网站 1:1 复刻（Axum 托管），默认监听全网段，局域网可直接访问。

## 本地运行（局域网）

```bash
cargo run -p cngokz-backend
# 默认监听 0.0.0.0:3000（含 192.168.0.101），局域网内访问：
#   http://192.168.0.101:3000/            网站（与 ~/cngokz-web 一模一样）
#   http://192.168.0.101:3000/api/v1/health  健康检查
# 仅本机：HOST=127.0.0.1 cargo run -p cngokz-backend
# 换端口：PORT=8080 cargo run -p cngokz-backend
# 换静态目录：WEB_DIR=/srv/cngokz/web cargo run -p cngokz-backend
```

## 分支说明

- `main`：稳定分支，只接受来自 `develop` 的合并，首次推送后建议设为默认分支并开启保护。
- `develop`：日常开发分支，所有功能分支都从这里切出、合回这里。

## 目录结构（骨架）

```text
cngokzapi/
├── backend/      # Rust 后端（Axum，对应 crate cngokz-backend）
├── web/          # 网站静态资源（与 ~/cngokz-web 一模一样，由后端托管）
├── docs/         # 文档（预留）
├── scripts/      # 脚本（预留）
├── migrations/   # 数据库迁移（预留）
├── PROJECT.md    # 项目规范与技术设计
├── CHANGELOG.md  # 开发日志
└── README.md
```

## 前端示例

网站原型在 `~/cngokz-web`（`index.html` / `styles.css` / `app.js`），正式开发时 `web/` 须与其视觉、布局、交互一模一样。

## 快速开始（推送到 GitHub）

```bash
# 本仓库已在本地初始化（main + develop），首次推送示例：
# 在 GitHub 建好空仓库后（不要勾选 README / .gitignore / License）：
git remote add origin <你的 GitHub 仓库地址>
git push -u origin main
git push -u origin develop

# 之后日常开发：
git checkout develop
git pull
git checkout -b feat/xxx
# ...开发...
git push -u origin feat/xxx
```

详细规范见 [PROJECT.md](PROJECT.md)，历史变更见 [CHANGELOG.md](CHANGELOG.md)。
