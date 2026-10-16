# cngokz-backend

Rust + Axum 模块化单体。层职责见 PROJECT.md §3.3，目录如下：

- `src/main.rs` / `app.rs` / `config.rs` / `state.rs` / `errors.rs`：入口、装配、配置、状态、统一错误
- `src/middleware/`：CORS（开发放行）、后续加 auth/限流
- `src/infrastructure/`：`db`（PG 懒连接）、`cache`（Redis Client），不写业务 SQL
- `src/modules/`：`health` / `leaderboard` / `records` / `bans`
  （`routes/handlers/service/repository/models/dto` 六件套，无数据访问可省 repository）
- `migrations/`：`sqlx migrate` 版本化 SQL（当前 `0001_init.sql`）
- `tests/`：集成测试（规划中，单测在各模块内）

## 命令

```bash
cargo test                              # 单测
cargo clippy --all-targets -- -D warnings
cargo run -p cngokz-backend             # 默认 0.0.0.0:3000，托管 frontend/dist，读本地 .env
curl http://127.0.0.1:3000/api/v1/health/ready   # {"status":"ready",...} 即 PG/Redis 双通
DATABASE_URL=... sqlx migrate run --source backend/migrations  # 应用迁移（测试库 62 已应用 0001）
```

无 PG/Redis 也可启动（懒连接），`/ready` 显示 degraded，数据接口仍走模拟层。
