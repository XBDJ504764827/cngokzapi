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
cargo run -p cngokz-backend             # 默认 0.0.0.0:3000，托管 frontend/dist
sqlx migrate run                        # 需先起 PG（DATABASE_URL）
```

无 PG/Redis 也可启动（懒连接），数据接口届时返回 503。
