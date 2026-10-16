# cngokzapi

CNGOKZ（中国 GOKZ 社区）官方网站 + 后端 API。

技术栈：前端 Vue 3 + TypeScript + Vite + Naive UI；后端 Rust + Axum；
数据库 PostgreSQL；缓存 Redis。规范见 [PROJECT.md](PROJECT.md)，变更见 [CHANGELOG.md](CHANGELOG.md)。

## 目录

```text
cngokzapi/
├── frontend/     # Vue 前端（见 frontend/README.md）
├── backend/      # Rust 后端（见 backend/README.md）
├── docs/         # 与 frontend/backend 同级：prototype/ 下是旧原型（只读参照）
├── scripts/      # 运维脚本（预留）
└── Cargo.toml    # workspace（成员：backend）
```

## 本地运行（局域网 192.168.0.101）

```bash
# 后端（默认 0.0.0.0:3000，无 PG/Redis 也可启动）
cargo run -p cngokz-backend
#   http://192.168.0.101:3000/              前端构建产物（需先 npm run build）
#   http://192.168.0.101:3000/api/v1/health 健康检查

# 前端开发（默认 0.0.0.0:5173，/api 代理到后端）
cd frontend && npm install && npm run dev
#   http://192.168.0.101:5173/
```

## 分支

- `main`：稳定分支，只接受来自 `develop` 的合并。
- `develop`：日常开发分支。
