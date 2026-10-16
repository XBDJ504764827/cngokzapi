# cngokzapi

CNGOKZ（中国 GOKZ 社区）后端 + 网站项目。

> 当前阶段：仓库骨架已初始化，等待首次推送到 GitHub 后再开始实际开发。

## 分支说明

- `main`：稳定分支，只接受来自 `develop` 的合并，首次推送后建议设为默认分支并开启保护。
- `develop`：日常开发分支，所有功能分支都从这里切出、合回这里。

## 目录结构（骨架）

```text
cngokzapi/
├── backend/      # Rust 后端（预留，推送后开始开发）
├── web/          # 网站（预留，需与 ~/cngokz-web 示例一模一样）
├── docs/         # 文档（预留）
├── scripts/      # 脚本（预留）
├── migrations/   # 数据库迁移（预留）
├── PROJECT.md    # 项目规范与技术设计
├── CHANGELOG.md  # 开发日志
└── README.md
```

`backend/`、`web/` 等目录目前只有占位文件，推送完成后才会放入真实代码。

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
