# 开发日志（CHANGELOG）

> 记录已经发生的事情：每次开发修改了什么、为什么修改、解决了什么问题、
> 修改了哪些文件、数据库和 API 有什么变化，以及什么时候完成。

格式约定：

- 按日期倒序追加，标题为 `## YYYY年MM月DD日 HH时mm分ss秒 主题`，时间统一使用**北京时间（UTC+8）**（2026年10月19日 00时48分 起启用；此前的历史条目沿用旧格式）。
- 每条写清：背景 / 变更内容 / 涉及文件 / 数据库与 API 变化 / 测试与结果。

---

## 2026年10月19日 18时10分10秒 SSE 页面秒级自动刷新（Phase 5）

- 背景：Phase 1-4 已把服务端数据更新压到 10s–10min，但浏览器仍需手动刷新/切换页面才能看到变化。
- 变更内容：
  - 后端新增 `EventBus`（tokio broadcast，256 缓冲）与 `GET /api/v1/stream?topics=`（Axum SSE，15s 心跳）。
    事件体 `{ topic, kind, id?, at }`，主题：`records` / `map:{id}` / `player:{steamid64}` /
    `bans` / `leaderboard` / `stats`。
  - 写入端发布事件：ingest 上报（records/map/player/stats）；`recent`（新成绩）、
    `record_updates`（改分/软删除）、`bans`（封禁 + 玩家失效）、`leaderboard`（榜单刷新）。
  - 前端新增 `composables/useLive.ts` 的 `useLiveReload(topics, reload, delay=400)`：
    EventSource 订阅 + 防抖刷新 + 卸载时关闭，断线由浏览器自动重连。
  - 接入页面：首页（stats，3s 防抖）、最新记录（records）、排行榜（leaderboard）、
    地图详情（map:{id}）、个人资料（player:{steamid64}，同时刷新地图进度）、封禁名单（bans）。
- 涉及文件：`backend/src/events.rs`、`backend/src/state.rs`、`backend/src/modules/stream/*`、
  `backend/src/app.rs`、`backend/src/modules/ingest/service.rs`、
  `backend/src/modules/sync/{jobs,worker}.rs`、`backend/Cargo.toml`、
  `frontend/src/composables/useLive.ts`、
  `frontend/src/features/{home/HomePage.vue,records/RecordsPage.vue,leaderboard/LeaderboardPage.vue,
  maps/MapDetailPage.vue,profile/ProfilePage.vue,bans/BansPage.vue}`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无表结构变更；新增 `GET /api/v1/stream`（SSE）。
- 测试与结果（北京时间）：`cargo fmt`/`build`/`clippy -D warnings`、`cargo test`（30 passed）、
  `npm run build` 全过；`curl -N` 验证收到 `records` / `map:961` 事件；
  Playwright 实测：地图详情在事件到达后自动重新拉取 `/api/v1/maps/961`（无跳转），
  最新记录与个人资料分别自动刷新 `/api/v1/records`、`/api/v1/players/{id}`；
  页面分别订阅 `stream?topics=records`、`map:961`、`player:7656…`；无控制台报错。
- 当前端到端延迟：自建服/第三方服写入 → 同步任务（10–30s）→ SSE → 页面 400ms 防抖后刷新；
  即页面最多滞后约 10–30s，且无需手动刷新。

## 2026年10月19日 17时47分44秒 实时增量同步 Phase 1-4：10s 成绩 / 30s 软删除 / 60s 封禁级联 / 10 分钟榜单

- 背景：数据虽已真实，但更新靠定时全量轮询（成绩 60s、封禁 1h、榜单 24h，且不处理删除/失效），
  需要在不依赖自建插件的前提下把延迟压到最短。
- 变更内容：
  - 新增 `sync_state` 游标表（迁移 `0009`）：`kztimer_recent_id` / `gokz_records_updated` /
    `gokz_bans_updated`，游标持久化、重启不丢。
  - `recent`（KZTimer）：60s → **10s**，改为单任务处理三模式，按 record id 游标只处理新记录，
    并对新增记录涉及的玩家重算本站汇总分（`RECENT_FETCH` 300→1000）。
  - 新增 `record_updates` 任务（gokz.top `/records?updated_since=&exclude_cheaters=false`，**30s**）：
    捕获新成绩、改分与**软删除（`is_valid=false`）**；新增 `upsert_record_state`，命中已有行时更新
    `is_valid/points`，兼容 KZTimer/gokz 双来源自然键去重。
  - `bans` 改为 `updated_since` 增量（**60s**）并**级联**：作弊封禁到达后把该玩家成绩标为无效、
    删除其 `player_scores`；迁移 `0010` 对已有封禁补一次（20 名玩家 / 3984 条成绩）。
  - `leaderboard`：24h → **10 分钟**；前 N 名玩家补全改为 6 小时内不重复入队（`enqueue_if_stale`）。
  - worker 新增 `RECORD_UPDATE_EVERY`，队列优先级仍为全局任务优先。
- 涉及文件：`backend/migrations/{0009_sync_state,0010_ban_cascade}.sql`、
  `backend/src/modules/sync/{mod,queue,worker,jobs,repository}.rs`、
  `backend/src/sources/gokz_top.rs`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：新增 `sync_state` 表；无公开 API 变化；上游新增 `/records?updated_since=`、
  `/bans?updated_since=` 调用。
- 测试与结果（北京时间）：`cargo fmt`/`build`/`clippy -D warnings`、`cargo test`（30 passed）通过；
  迁移已应用并重启后端；实测三个游标持续推进，`record_updates` 每 30s 处理 3–12 条无报错，
  `kztimer_recent_id` 已推进到 `27818608`；封禁级联后「封禁玩家有效成绩 / 榜单汇总」均为 0；
  新成绩已能在 10–30s 内出现在 `/api/v1/records` 与地图榜。
- 当前真实延迟：新成绩 10–30s、软删除/封禁 30–60s、排行榜 ≈10 分钟、服务器 5 分钟；
  自建插件事件直推接入后可到 <1s（Phase 5+，本次未做）。

## 2026年10月19日 16时53分13秒 真实数据回填完成 + 排行榜/最新记录接入真实头像

- 背景：`player_profile` 回填已跑完，同时发现排行榜与最新记录前端虽然拿到了 `avatar` 字段，
  但仍调用 `avatarUrl(steamId64)` 忽略真实头像，页面显示为 DiceBear 占位图。
- 变更内容：
  - 排行榜 `LeaderboardPage` 改用 `avatarUrl(e.steam_id64, e.avatar)`。
  - 最新记录后端 `RecordEntry` 新增 `avatar`（SQL 联查 `players.avatar_url`），前端改用真实头像。
  - 封禁名单 SQL 改为 `COALESCE(bans.avatar_url, players.avatar_url)`，封禁玩家也显示真实头像。
  - 修复 `PlayerCell`（地图详情 / 封禁名单）头像永远显示首字母的问题：NAvatar 存在 default slot
    时会优先渲染文字而忽略 `src`；改为 `#fallback` 插槽 + `fallback-src`，真实头像正常渲染。
  - 补跑 18 个因旧错误/瞬时 404 未写入头像的档案任务；确认剩余 6 人上游 `avatar_hash` 为空。
- 涉及文件：`backend/src/modules/records/{models,repository}.rs`、
  `backend/src/modules/bans/repository.rs`、`frontend/src/components/PlayerCell.vue`、
  `frontend/src/features/{leaderboard/LeaderboardPage.vue,records/{RecordsPage.vue,types.ts}}`、
  `PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：`GET /api/v1/records` 新增 `avatar` 字段（向后兼容，前端类型同步）。
- 测试与结果（北京时间）：`cargo fmt`/`build`/`clippy -D warnings`、`cargo test`（30 passed）、
  `npm run build` 全过；重启后端后 Playwright 实测排行榜 **150/150**、最新记录 **60/60**、
  地图详情 40/40、封禁名单 20/20 均使用真实头像；数据审计：玩家 3994 人，头像 3988 人
  （99.85%，6 人上游无头像）、国籍 3024 人；记录 32.6 万、地图 1046（963 有制作人）、
  在线服务器 619、封禁 503。

## 2026年10月19日 16时19分42秒 真实数据补全：全量回填玩家头像/国籍 + 同步吞吐调整

- 背景：审计发现所有页面已读真实数据，但 `players` 表 3994 人中仅 163 人有头像、131 人有国籍——
  排行榜与地图榜大量玩家退化为 DiceBear 占位图；需把缺失的真实头像/国家全部补齐。
- 变更内容：
  - 新增轻量同步类型 `player_profile`：只请求 GOKZ.TOP `GET /players/{steamid64}`
    （名称/头像 hash/国家），写回后即得到 Steam CDN 真实头像与国旗，不重拉成绩。
  - 迁移 `0008_player_profile_backfill.sql`：为所有缺头像或国籍的玩家入队该任务（约 3800 人，
    按 400ms 限速在后台回填）。
  - worker 吞吐上调：`BATCH` 4→8、`MAX_INFLIGHT` 3→4、`TICK` 5s→2s（外部限速闸门不变，仍礼貌回源）。
  - 修复 `upsert_player_profile` 在无头像时写入 `NULL` 导致 `players.avatar_url NOT NULL` 报错的问题
    （改为空串，仍保留「空值不覆盖已有头像」语义）。
  - 队列优先级把 `player_profile` 与 `player` 归为一类，避免玩家任务饿死全局同步。
- 涉及文件：`backend/src/modules/sync/{mod,queue,worker,jobs,repository}.rs`、
  `backend/migrations/0008_player_profile_backfill.sql`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无表结构变更（`0008` 仅入队任务）；无公开 API 变化。
- 测试与结果（北京时间）：`cargo fmt`/`build`/`clippy -D warnings` 通过；迁移已应用并重启后端；
  排行榜三模式 **200/200** 玩家已有真实头像；头像总数从 163 持续回填中，后台任务继续跑完；
  8 张带旧错误的任务会在退避后重试。地图制作人 963/1046（其余 83 张 kzgo 上游无数据）、
  首页公告为运营静态文案，均非可获取的接口数据。

## 2026年10月19日 15时14分26秒 个人资料：最近完成地图 + 全部已完成/未完成地图 + 搜索框优化

- 背景：个人资料只展示最近 20 条成绩，看不到地图完成全貌；且顶级玩家旧同步只拉最近 1000 条成绩，
  导致完成地图数严重偏少（Flonnych KZT 本地仅 150 张，上游 `maps_completed` 为 955）。
- 变更内容：
  - 后端：
    - `sources/gokz_top.rs` 新增 `player_pbs`（`GET /records/pb`，模式 + 类型 NUB/PRO + 分页）。
    - 玩家同步 `run_player` 改为按模式分页拉 PB（主关，每页 1000 / 上限 20000）并幂等落库，
      再拉最近成绩；抽出 `upsert_top_records` 复用写入逻辑。
    - 新增 `GET /api/v1/players/{steam_id64}/maps?mode=&status=completed|uncompleted&page=&per_page=&q=`：
      按模式统计主关完成情况，返回分页数据 + `total / completed_total / maps_total`；
      已完成项带最佳耗时与分数，未完成项仅地图信息；支持地图名模糊搜索。
    - 迁移 `0007_player_pb_backfill.sql`：重置 `player` 任务，回填全部完成地图。
  - 前端：
    - 个人资料「最近成绩」更名为「最近完成地图」（内容不变）。
    - 新增「地图进度」区块：KZT/SKZ/VNL 切换、「已完成 / 未完成」标签（带计数）、地图名搜索、
      每页 24 张地图卡片（缩略图 + 名称 + 当前模式 Tier + 最佳耗时/分数），支持分页。
    - 顶部玩家搜索框重做为大号搜索卡：顶部主色强调线、搜索图标、主按钮、格式提示与「高级查询」入口。
- 涉及文件：`backend/src/sources/gokz_top.rs`、`backend/src/modules/sync/jobs.rs`、
  `backend/src/modules/players/{models,dto,repository,service,handlers,routes}.rs`、
  `backend/migrations/0007_player_pb_backfill.sql`、
  `frontend/src/features/profile/{ProfilePage.vue,api.ts,types.ts}`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无表结构变更（`0007` 仅重置任务）；新增 `GET /api/v1/players/{steam_id64}/maps`；
  玩家同步新增外部调用 `/records/pb`。
- 测试与结果（北京时间）：`cargo fmt`/`build`/`clippy -D warnings`、`cargo test`（**30 passed**）、
  `npm run build` 全过；迁移已应用并重启后端；实测 Flonnych KZT 完成地图 150 → 958、
  SKZ 84、VNL 428（与上游基本一致）；接口 KZT 已完成 958 / 未完成 87 / 总计 1045，
  搜索 `sunstone` 命中 1 张；Playwright 验证模式/状态切换、搜索、分页与 390px 无横向滚动均正常。

## 2026年10月19日 14时51分06秒 背景留白调和：暖纸底改低饱和暖灰、去掉全页渐变

- 背景：上一版暖纸底色（`#F6F3ED` + 纵向渐变）偏黄/偏橘，大面留白与朱砂红主色互相抢眼，
  且顶部与底部双色渐变显得不干净。
- 变更内容：
  - `styles/global.css`：`--bg-page` 由 `#F6F3ED` 改为低饱和暖灰 `#F1F0ED`，同步下调
    `--bg-subtle/--bg-hover/--tint-panel/--row-hover/--border-color/--border-strong` 的棕色调；
    `body` 删除纸感纵向渐变，改为平涂底色。
  - `app/theme.ts`：`bodyColor/hoverColor/borderColor/dividerColor/tableColorHover` 同步改为中性暖灰，
    文字色由棕调改为中性灰棕，卡片仍为白色（靠边框 + 微阴影分层）。
- 涉及文件：`frontend/src/styles/global.css`、`frontend/src/app/theme.ts`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无。
- 测试与结果（北京时间）：`npm run build` 通过；Playwright 1440 / 980 / 390 全站 10 页无报错、
  无新增横向滚动；白底卡片与背景留白对比舒适，朱砂红主色仍保持鲜明。

## 2026年10月19日 14时44分03秒 全站色彩体系重做：暖调朱砂红 + 松绿/赭石/石板蓝（平涂无渐变）

- 背景：原设计为「白底 + 单一墨绿主色」，长时间浏览易审美疲劳；首次多色方案（靛蓝/紫罗兰渐变）
  被反馈「AI 感过重」，故重做为暖调、平涂、无渐变的编辑风配色。
- 变更内容：
  - `styles/global.css` 重写 token：主色为朱砂红 `#C1432B`，辅助色松绿 `#1F6F6B` / 赭石 `#B07D2A` /
    石板蓝 `#4B6472`；语义色 `--success/--info/--danger`；三模式色 `--mode-kzt/skz/vnl`（朱砂/松绿/赭石）
    与工具类 `.mode-*`；色调工具类 `.tone-rust/.tone-teal/.tone-ochre/.tone-slate`；
    难度色 8 档 `--tier-1`～`--tier-8` 改为土色系；奖牌改为平涂 `--medal-*`；背景为暖纸感纵向渐变。
  - `app/theme.ts` 同步新主色/成功/警告/错误/信息色，表头改浅朱砂底，按钮聚焦环改朱砂。
  - **移除全部渐变与多色「极光」晕染**：导航当前项、主按钮、Logo、Hero 主按钮、WR 徽章、
    视图切换、区块标题竖条、奖牌均改为平涂；Hero 只保留极浅的朱砂单色晕染。
  - 各页按模式/语义着色：首页 4 张统计卡分用朱砂/松绿/赭石/石板蓝；排行榜三列、最新记录三列、
    个人资料三模式卡与最近成绩列、地图详情三模式榜单与模式切换均按 KZT/SKZ/VNL 着色。
- 涉及文件：`frontend/src/styles/global.css`、`frontend/src/app/theme.ts`、
  `frontend/src/layouts/DefaultLayout.vue`、`frontend/src/components/RankBadge.vue`、
  `frontend/src/features/home/HomePage.vue`、`frontend/src/features/leaderboard/LeaderboardPage.vue`、
  `frontend/src/features/records/RecordsPage.vue`、`frontend/src/features/profile/ProfilePage.vue`、
  `frontend/src/features/maps/{MapsPage,MapDetailPage}.vue`、
  `frontend/src/features/servers/ServersPage.vue`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无。
- 测试与结果（北京时间）：`npm run build` 通过；Playwright 核对 1440 / 980 / 390 三档，
  全站 10 个页面无控制台报错、无新增横向滚动；首页/榜单/地图/地图详情/服务器/记录/资料/封禁/玩家/登录
  均按新配色渲染，三模式颜色区分明确、难度徽章 8 档可辨；`/records`、`/bans` 在 390px 的横向溢出
  为改版前已有问题（已用 git stash 对比确认，与本次配色无关）。

## 2026年10月19日 14时15分41秒 个人资料页重设计：身份卡 + 三模式等级档案（国旗 / 头像 / 评级）

- 背景：个人资料页此前是「头像英雄条 + 模式积分表格」，信息密度低、表格单调；用户要求突出头像、
  SteamID64、所在国家国旗，以及 KZT/SKZ/VNL 三模式的总积分、全球排名与等级，且等级判定与
  GOKZ.TOP 一致。
- 变更内容：
  - 后端：
    - `sources/gokz_top.rs` 新增 `GET /v1/players/{steamid64}` 客户端（`name` / `avatar_hash` /
      `country`），`ProfileRow.avatar_url()` 拼出 Steam CDN 全尺寸头像；新增解析单测。
    - 玩家同步 `run_player` 先拉基础档案（第 0 步），写回名称 / 头像 / 国籍。
    - `sync/repository.rs` 新增 `upsert_player_profile`（空值不覆盖已有值）；
      `players` 增列 `country`（迁移 `0006_player_country.sql`），同时把 `player` 任务重置为
      `pending` 回填历史玩家的国家与头像。
    - `players` 模型与仓库增 `country`，个人资料接口返回。
  - 前端：
    - 身份卡：真实头像（国旗角标）、玩家名、可复制 SteamID64、中文国家名与区域 chip、Steam 主页入口。
    - 三模式档案卡：全球总积分、全球排名、等级（New 1.00 → Legend 10.00）与 10 级进度条、
      「升到 X 还需 Rt」；`rating < 1` 或缺失显示「未评级」；未上榜模式占位。
      等级与 Rating 算法在 `features/profile/level.ts`（纯函数，三模式共用）。
    - 国旗用 `flagcdn` 图片 + `Intl.DisplayNames` 中文名（`utils/country.ts`），图片失败时只显示文字。
    - 查询卡改为与玩家查询页一致的搜索面板；响应式：模式卡 3 → 2 → 1 列，窄屏身份卡纵向排列。
    - 最近成绩区块及样式保持原样。
- 涉及文件：`backend/src/sources/gokz_top.rs`、`backend/src/modules/sync/{jobs,repository}.rs`、
  `backend/src/modules/players/{models,repository,service}.rs`、
  `backend/migrations/0006_player_country.sql`、
  `frontend/src/features/profile/{ProfilePage.vue,types.ts,level.ts}`、
  `frontend/src/utils/country.ts`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：新增迁移 `0006_player_country.sql`（`players.country` + 重置 `player` 任务）；
  `GET /api/v1/players/{steam_id64}` 新增 `country` 字段，`avatar` 开始返回真实头像地址。
- 测试与结果（北京时间）：`cargo fmt`/`build`/`clippy -D warnings`、`cargo test`（**30 passed**）、
  `npm run build` 全过；迁移已应用，实测 Flonnych 写回 `country=UA` 与 Steam 头像；
  页面核对 KZT Legend Rt 10.98 #2 / SKZ Regular Rt 5.66 #495 / VNL Master Rt 9.35 #62，
  2 模式玩家显示「该模式暂无榜单记录」，`rating=0` 显示「未评级」；
  Playwright 1440 / 980 / 390 三档无横向滚动、无控制台报错。

## 2026年10月19日 13时47分49秒 玩家查询页重新设计：实时格式识别 + 示例 + 最近查询

- 背景：玩家查询页此前只有一个输入框和一行格式列表，样式单调、缺少引导，也无法即时判断输入是否可识别。
- 变更内容：
  - 重做 `PlayersPage.vue`：
    - Hero 标题区（eyebrow + 标题 + 说明），搜索卡改为视觉主体（主色顶栏、搜索图标、大号主操作按钮）。
    - 实时识别条：输入时显示识别出的格式与推算出的 SteamID64 预览；无法识别时输入框转 warning 态并提示。
    - 4 个示例 chip（SteamID64 / STEAM_x:y:z / SteamID3 / 主页链接），点击即填入并查询。
    - 最近查询：成功解析后写入 `localStorage`（`cngokz:recent-player-queries`，按 SteamID64 去重、最多 6 条），
      展示头像 / SteamID64 / 格式 / 时间，支持一键清除与点击回到资料页。
    - 格式说明改为 2×2 卡片（图标 + 说明 + 示例），点击即用示例查询；右侧新增
      「如何找到自己的 SteamID」步骤与提示卡。
    - 响应式：≤980px 主栅格单列，≤600px 搜索框、格式卡与最近查询纵向排列；
      同时修正 Hero 晕染负边距在窄屏造成的横向滚动。
  - 接入此前已存在但未被使用的 `players/detect.ts` 做前端实时识别（最终解析仍以后端接口为准）。
- 涉及文件：`frontend/src/features/players/{PlayersPage.vue,detect.ts}`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无；仍使用 `GET /api/v1/players/resolve?q=`。
- 测试与结果（北京时间）：`npm run build` 通过；Playwright 验证 1440 / 980 / 390 三档均无横向滚动、无控制台报错；
  `STEAM_1:1:168575119` 实时识别并预览 `76561198297415967`，非法输入显示警示；
  点击示例跳转 `/profile/76561198297415967`，最近查询的写入 / 清除 / 回跳均正常。

## 2026年10月19日 05时42分54秒 导航新增「玩家」查询（多格式 SteamID 解析）

- 背景：用户要求顶部导航新增玩家功能，可通过 SteamID / SteamID64 / Steam 个人主页链接查询玩家资料。
- 变更内容：
  - 后端新增 `GET /api/v1/players/resolve?q=`：支持 17 位 SteamID64、`STEAM_x:y:z`、`[U:1:w]` /
    纯数字账号 id、`steamcommunity.com/profiles/<id>`、`/id/<vanity>`（后者用 Steam Web API
    `ResolveVanityURL` + `STEAM_API_KEY`）；返回 `{ steam_id64 }`，非法输入 400 / 未找到 404。
  - 前端新增 `/players` 页面与导航「玩家」入口：输入任意格式 → 解析 → 跳转 `/profile/:steamid64`；
    个人资料页查询框也复用同一解析接口（不再只接受 17 位）。
  - `sources/steam.rs` 新增 `resolve_vanity`；`players` 新增 `dto.rs` 与解析单测。
- 涉及文件：`backend/src/modules/players/{dto,models,service,handlers,routes,mod}.rs`、
  `backend/src/sources/steam.rs`、`frontend/src/features/players/{api.ts,PlayersPage.vue}`、
  `frontend/src/router/index.ts`、`frontend/src/layouts/DefaultLayout.vue`、
  `frontend/src/features/profile/ProfilePage.vue`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无迁移；新增 `GET /api/v1/players/resolve`。
- 测试与结果（北京时间）：`cargo fmt`/`build`/`clippy -D warnings`、`cargo test`（**29 passed**）、
  `npm run build` 全过；实测 6 种输入均正确（含 `/id/gaben` → `76561197968052866`）；
  Playwright 从 `/players` 提交 `STEAM_1:1:168575119` 跳转 `/profile/76561198297415967`，导航显示「玩家」。

## 2026年10月19日 05时33分32秒 个人资料最近成绩按模式分组 + 地图缩略图/详情跳转

- 背景：个人资料「最近成绩」混合三种模式且无地图图片，不便于辨认；用户要求按模式分三类展示，
  显示地图图片，并支持点击地图名查看地图详情。
- 变更内容：
  - 后端 `players`：`PlayerRecordEntry` 增 `map_id`；`recent_records` 改为按模式查询
    （`mode` + `points > 0`，每模式 20 条）；service 循环三模式拼接，避免单一模式占满列表。
  - 前端 `ProfilePage` 重做：查询卡独立；英雄卡 + 模式积分表；最近成绩分 KZT/SKZ/VNL 三列，
    每条含地图缩略图（缩略图与地图名均可点击进 `/maps/:id`）、关卡（主关/Bn）、完成时间、
    分数、日期；无成绩的模式显示「暂无有分成绩」；≤1080px 堆叠。
  - 地图缩略图走 `utils/mapImage.ts` 的 `thumbnails`，缺图有占位。
- 涉及文件：`backend/src/modules/players/{models,repository,service}.rs`、
  `frontend/src/features/profile/{types.ts,ProfilePage.vue}`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无迁移；`GET /api/v1/players/{steam_id64}` 的 `recent_records`
  每项新增 `map_id`，内容改为每模式最多 20 条有分成绩。
- 测试与结果（北京时间）：`cargo fmt`/`build`/`clippy -D warnings`、`npm run build` 全过；
  实测 Flonnych：KZT 20 / SKZ 0 / VNL 2 条，0 分 0 条；Playwright 确认三列、22 个地图链接，
  点击地图名跳转 `/maps/44`。

## 2026年10月19日 05时27分57秒 过滤无分成绩（最新记录 / 个人资料最近成绩）

- 背景：最新记录与个人资料「最近成绩」中出现大量 0 分成绩（玩家未打破自己的记录 / 未进入计分范围，
  上游给 0 分），用户要求这些不计分完成不再统计与展示。
- 变更内容：
  - `GET /api/v1/records`（最新记录）：SQL 增加 `AND r.points > 0`，只返回有分成绩。
  - 个人资料：`recent_records` 增加 `AND r.points > 0`；「已同步成绩」计数（`player_scores` 子查询）
    同样只 `count(*) ... AND points > 0`。
  - 地图详情 PRO/TP 榜保持原样（仍可展示无分但最快的成绩，0 分显示「—」）。
- 涉及文件：`backend/src/modules/records/repository.rs`、
  `backend/src/modules/players/repository.rs`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无迁移；两个只读接口的返回内容变更为仅含有分记录。
- 测试与结果（北京时间）：`cargo fmt`/`cargo build`/`cargo clippy -D warnings` 通过；
  实测 `/api/v1/records` 60 条中 0 分 = 0（最低 236 分）；
  个人资料最近成绩 0 分 = 0，KZT 已同步成绩计数 186（仅计有分记录）。

## 2026年10月19日 05时19分21秒 修复 PlayerCell 玩家名不跳转（Boolean prop 默认值陷阱）

- 背景：地图详情 PRO/TP 榜点击玩家名没有跳转到个人资料。
- 原因：`PlayerCell` 的 `link?: boolean` 是 Boolean 类型 prop，未传时 Vue 会将其转为 `false`
  （不是 `undefined`），导致 `v-if="link !== false"` 不成立，名称渲染为纯文本；
  封禁页等所有未传 `link` 的调用同样受影响。
- 修复：改用 `withDefaults(defineProps<...>(), { link: true, compact: false })`，
  模板改为 `v-if="link"`；`compact` 仍是可选。
- 涉及文件：`frontend/src/components/PlayerCell.vue`、`PROJECT.md`、`CHANGELOG.md`。
- 测试与结果（北京时间）：`npm run build` 通过；Playwright 验证：
  地图详情点击玩家名跳转 `/profile/76561198325578948`；封禁页 20 个、排行榜 150 个玩家名均为链接。

## 2026年10月19日 01时17分28秒 排行榜隐藏 SteamID64；地图详情 PRO/TP 双列并排

- 背景：用户要求（1）排行榜可点击玩家名进资料页、不显示 SteamID64；（2）地图详情 PRO/TP
  上下排列看得累，改为双列，且不能出现横向滑动。
- 变更内容：
  - 排行榜：移除行内 SteamID64（搜索仍支持 SteamID64）；玩家名仍为 `/profile/:steamid` 链接，
    点击已验证跳转正常。
  - 地图详情：PRO/TP 两榜改为**双列并排**（`repeat(2, minmax(0,1fr))`，≤1100px 堆叠）；
    `PlayerCell` 新增 `compact` 模式（隐藏 SteamID64、头像 26px）；上传时间改 `YY-MM-DD HH:mm`，
    列宽收紧（排名 56 / 完成时间 84 / 分数 52 / 存读点 58 / 上传 126），实测两表
    `scrollWidth == clientWidth`（无横向滑动），所有字段完整可见。
- 涉及文件：`frontend/src/components/PlayerCell.vue`、
  `frontend/src/features/leaderboard/LeaderboardPage.vue`、
  `frontend/src/features/maps/MapDetailPage.vue`、`frontend/src/utils/format.ts`、
  `PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无。
- 测试与结果（北京时间）：`npm run build` 通过；Playwright 验证：排行榜可见 SteamID64 数量 = 0，
  点击首个玩家名跳转 `/profile/76561198297415967`；地图详情双列 `568px 568px`，
  两张表 `client=564 / scroll=564`，无横向溢出；截图确认列完整显示。

## 2026年10月19日 01时07分29秒 地图详情页改版：三模式 PRO/TP 榜 + 大图/等级/制作人/最快时间

- 背景：删除全局模式切换后，地图详情默认只能看 KZT；用户要求三模式都可查看并重新设计，
  硬性要求：大图、KZT/SKZ/VNL 等级、制作人、最快时间（PRO/TP）、前 100 名头像/名称/完成时间/
  分数/上传时间、PRO/TP 排行榜、TP 存读点数量。
- 变更内容：
  - 后端 `modules/maps`：`MapRecordEntry` 增 `teleports` / `created_at`（上传时间）；新增 `ModeBoard`；
    `MapDetail` 改为 `{ map, boards[], syncing }`，一次返回 KZT/SKZ/VNL 三模式。
    `board_records` 按 `(map_id, mode, stage=0, is_valid)` + `teleports=0`（PRO）/`>0`（TP）分榜，
    同一玩家在各自分类取最佳、前 100；`GET /api/v1/maps/{id}` 不再需要 `mode` 参数，
    首次访问对未同步模式入队。
  - 前端 `MapDetailPage` 重做：大图 + 名称/工坊/制作人/大小/Bonus + 三模式等级 chips；
    「三模式最快完成」同时展示 PRO/TP；页内 KZT/SKZ/VNL 切换；PRO/TP 两张全宽排行榜
    （排名徽章、头像、名称、完成时间、分数、上传时间，TP 多一列存读点），上下堆叠避免截断。
  - `utils/format` 新增 `formatDateTime`（带年份），地图详情上传时间使用。
- 涉及文件：`backend/src/modules/maps/{models,repository,service,handlers,dto}.rs`、
  `frontend/src/features/maps/{types.ts,api.ts,MapDetailPage.vue}`、`frontend/src/utils/format.ts`、
  `PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无迁移；`GET /api/v1/maps/{id}` 响应改为 `{ map, boards, syncing }`（breaking）。
- 测试与结果（北京时间）：`cargo clippy -D warnings`、`cargo test`（28 passed）、`npm run build` 全过；
  实测 `kz_catalyst_gfix` 三模式 PRO/TP 均有数据、TP#1 含存读点与上传时间；
  截图确认大图/等级/制作人/最快时间/两榜列完整显示。

## 2026年10月19日 00时48分37秒 CHANGELOG 标题格式改为「YYYY年MM月DD日 HH时mm分ss秒 主题」

- 背景：用户希望更精准地按日期与时间查询变更，并指定中文时间格式。
- 变更：标题格式由 `## YYYY-MM-DD 主题` 改为
  `## YYYY年MM月DD日 HH时mm分ss秒 主题`；时间统一使用**北京时间（UTC+8）**。
  服务器系统时区为 UTC，因此北京时间比系统 `date` 快 8 小时（如系统 16:48 → 北京次日 00:48）。
  本次及以后的新条目使用新格式，历史条目保留原样。
- 涉及文件：`CHANGELOG.md`。

## 2026年10月19日 00时44分00秒 说明并优化「0 分」显示（KZTimer 前 1000 名计分）

- 背景：最新记录中出现部分玩家完成地图后分数为 0，用户询问原因。
- 结论：**不是 bug**。KZTimer 只对每张地图/关卡的前 1000 名计分，超出范围的完成仍会被记录，
  但 `points = 0`；同一玩家更慢的非 PB 完成也常落在此范围。
  实测验证：`fou` 于 `bkz_goldbhop_csgo` 本站记录 100.078s=0 分，其 PB 80.141s=786 分；
  `kz_carp_v2` 前 1000 名最慢 59.65s，Green 的 74.164s/99.164s 在范围外。
- 变更：最新记录的 0 分改显示为「未计分」（灰色，hover 说明规则）；
  个人资料「最近成绩」与地图详情榜的单条 0 分显示为「—」。
- 涉及文件：`frontend/src/features/records/RecordsPage.vue`、
  `frontend/src/features/profile/ProfilePage.vue`、`frontend/src/features/maps/MapDetailPage.vue`、
  `PROJECT.md`、`CHANGELOG.md`。
- 测试与结果：`npm run build`（含 `vue-tsc -b`）通过。

## 2026-10-18 修复排行榜三模式列未对齐

- 背景：三模式排行榜的第 2/3 列（SKZ / VNL）整体下沉 16px，三列未对齐。
- 原因：全局 `.panel + .panel { margin-top: 16px }`（为堆叠面板设计）也命中了 grid 同行的兄弟面板。
- 修复：`LeaderboardPage.vue` 增加 `.lb-panel + .lb-panel { margin-top: 0 }`
  （scoped 选择器优先级更高，不影响地图详情/档案页的堆叠面板间距）。
- 测试与结果：`npm run build` 通过；Playwright 量测三列 `panel/head/首行` 的 top
  均为 `253.8 / 254.8 / 309.8`，截图确认完全对齐。

## 2026-10-18 移除导航模式切换 + 排行榜三模式同页并排

- 背景：用户要求删除导航栏的模式切换功能（以后不用它切模式）；原排行榜只显示单个模式，
  需改为同时展示 KZT/SKZ/VNL 三模式排行，且不遮挡数据。
- 变更内容：
  - 导航栏移除 KZT/SKZ/VNL 切换按钮及 `.mode-switch/.mode-dot` 样式；`stores/app.ts` 简化为
    固定 `currentMode='KZT'`（仅供地图页沿用），删除 `setMode`/`cycleMode`。
  - 排行榜重做：一次并行拉取三模式榜单，**三列并排**（KZT / SKZ / VNL），每列前 50 名；
    行内两行式展示名次徽章、头像、玩家名（链接资料页）、总分、完成地图/记录数/平均分/最近游玩，
    全部字段可见不截断；搜索 / 活跃度 / 排序同时作用于三列；≤1080px 自动堆叠为单列。
- 涉及文件：`frontend/src/layouts/DefaultLayout.vue`、`frontend/src/stores/app.ts`、
  `frontend/src/features/leaderboard/LeaderboardPage.vue`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无（复用 `GET /api/v1/leaderboard?mode=` 三次请求）。
- 测试与结果（2026-10-18）：`npm run build`（含 `vue-tsc -b`）通过；
  Playwright 1440 截图核对三列排行完整展示，导航栏已无模式切换按钮。

## 2026-10-18 KZTimer 实时成绩 + 双来源服务器（插件优先 + Steam Web API）

- 背景：用户要求（1）最新记录改从 KZTimer 官方接口实时获取，不依赖 gokz.top；
  （2）服务器列表两种来源——装了本站插件的服务器上报实时信息、或 Steam 公开 API 获取，
  两者命中同一服务器时插件优先；（3）网站只显示 KZ 模式的 CS 服；
  （4）必要数据（如地图信息）可先从 gokz.top 拉入数据库缓存，定时刷新，但不能完全依赖 gokz.top。
- 变更内容：
  - 新增 `sources/kztimer.rs`：KZTimer Global API 客户端（`/records/top/recent`、`/records/top`），
    含模式映射（`kz_timer`/`kz_simple`/`kz_vanilla`）与无时区 UTC 时间解析。
  - `run_recent` 改走 KZTimer 广播式最新成绩（本地按模式筛选），`source='kztimer'`，
    轮询加密到 **60s**（近实时）；`external_id = kztimer:{id}`。
  - 新增 `sources/steam.rs`：Steam Web API `IGameServersService/GetServerList`（需 `STEAM_API_KEY`），
    按 `appid 730/4465480` × `gametype ∈ {gokz,cs2kz,kz}` 多次查询并按 `ip:port` 去重。
  - 服务器改为双来源：`run_servers` 走 Steam（每 5 分钟，先置离线再写回在线）；
    新增插件上报 `POST /api/v1/ingest/servers`（`X-ApiKey`，复用 `server_keys` 鉴权）；
    两源以 `(ip, port)` 合并，插件（10 分钟内）优先不被 Steam 覆盖。
  - 迁移 `0005_server_sources.sql`：`servers` 增 `source/appid/game_mode/steam_id/reported_at`，
    新增唯一索引 `(ip, port)`；`GET /api/v1/servers` 只返回在线 KZ CS 服并附带 `source`。
  - 运维：`ingest-key delete-server <ip> <port>`。
  - 前端：服务器卡片/列状显示数据来源（插件 / Steam）标签。
  - `config.rs` 增 `STEAM_API_KEY`（`.env` 已填，`.env.example` 留占位）。
- 涉及文件：新增 `backend/src/sources/{kztimer,steam}.rs`、`backend/migrations/0005_server_sources.sql`；
  修改 `backend/src/{config.rs,app.rs}`、`backend/src/sources/mod.rs`、
  `backend/src/modules/sync/{jobs,worker,repository}.rs`、
  `backend/src/modules/ingest/{dto,service,handlers,routes}.rs`、
  `backend/src/modules/servers/{models,repository}.rs`、`backend/src/bin/ingest-key.rs`、
  `frontend/src/features/servers/{types.ts,ServersPage.vue}`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：迁移 `0005` 已应用；`GET /api/v1/servers` 增 `source`；
  新增 `POST /api/v1/ingest/servers`。
- 测试与结果（2026-10-18）：
  - `cargo fmt`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（**28 passed**，含 KZTimer 映射/URL 单测）、
    `npm run build`（含 `vue-tsc -b`）全过。
  - 实测：Steam 同步 **529 台 KZ 服**（含实时人数/地图）；`/api/v1/records` 最新条目为 KZTimer 源且滞后 < 1 分钟；
    插件上报无 key=401、合法=200，且 `127.0.0.1:27015` 以 `source=plugin` 带玩家名单显示（已用 `delete-server` 清理）。
  - 遗留（已在 §9.2 说明）：KZTimer `player_ranks` 的 KZT 恒 500，故地图元信息/排行榜/封禁仍
    以 gokz.top **落库缓存**（启动全量 + 定时刷新，不阻塞展示）；后续可迁到 KZTimer 或本站自建计分。

## 2026-10-18 最新记录展示地图图片/地图名/头像/分数/耗时/上传时间，地图可点进详情

- 背景：用户反馈最新记录仍不能完整展示，需同时显示地图名称、地图图片、玩家头像、
  获取分数、完成地图时间、记录上传时间，并要求点击地图可看地图详情。
- 变更内容：
  - 后端 `modules/records`：`RecordEntry` 新增 `map_id`（跳地图详情/拼图片），
    `date` 改为 `created_at`（UTC ISO8601，前端按本地时区格式化）；SQL 加选 `m.id`、
    `to_char(created_at AT TIME ZONE 'UTC', ...)`。
  - 前端 `RecordsPage` 每条改为「缩略图 + 两行」：左侧地图缩略图（`mapImage` thumbnails，
    点击进 `/maps/:id`），右侧第一行 地图名（可点）+ 完成耗时，第二行 玩家头像 + 玩家名（可点）
    + `N 分` + 上传时间（`MM-DD HH:mm`，hover 看完整时间）；一列内完整展示，无横向滚动。
  - `utils/format.ts` 新增 `formatDateTimeShort`；`features/records/types.ts` 同步字段。
- 涉及文件：修改 `backend/src/modules/records/{models,repository}.rs`、
  `frontend/src/features/records/{types.ts,RecordsPage.vue}`、`frontend/src/utils/format.ts`、
  `PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无迁移；`GET /api/v1/records` 新增 `map_id`，`date` 字段被 `created_at`
  （ISO8601）取代。
- 测试与结果（2026-10-18）：`cargo fmt`/`cargo build`、`npm run build` 通过；
  实测接口返回 `map_id`/`created_at`；Playwright 截图确认六项信息齐全，点击缩略图正确跳转
  `/maps/285`。

## 2026-10-18 服务器按钮左右分列 + 最新记录两行式完整展示

- 背景：用户反馈服务器卡片上「加入游戏 / 复制 IP」挤在一起，要求分列到最左/最右；
  最新记录三列每列宽度不够，需要横向滚动才能看全，要求一列内完整展示。
- 变更内容：
  - 服务器卡片 `.server-actions` 改 `justify-content: space-between`：主操作「加入游戏」靠左、
    次操作「复制 IP」靠右（列状视图不受影响）。
  - 最新记录由 `NDataTable`（5 列固定宽 + `scroll-x`，需横向滚动）改为**两行式自绘条目**：
    第一行 头像 + 玩家名（墨绿链接）+ 耗时（墨绿等宽）；第二行 地图 + `N 分` + 日期。
    一列内即可完整展示玩家/地图/时间/分数/日期，去除横向滚动；超长名称省略号 + hover 全称。
- 涉及文件：修改 `frontend/src/features/servers/ServersPage.vue`、
  `frontend/src/features/records/RecordsPage.vue`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无。
- 测试与结果（2026-10-18）：`npm run build`（含 `vue-tsc -b`）通过；Playwright 1440 截图核对：
  卡片按钮左右分列；最新记录三列各 20 条两行式完整显示，无横向滚动。

## 2026-10-18 服务器「加入游戏」按钮 + 全站视觉强化

- 背景：用户反馈服务器只有「复制 IP」一个按钮，要求增加拉起 Steam 加入游戏的按钮
  （CS 游戏 AppID 更新为 4465480）；同时反馈网站太素、按钮与玩家名一样不明显，要求优化样式。
- 变更内容：
  - 服务器：卡片/列状视图都新增「加入游戏」主按钮，点击 `window.location.href` 到
    `steam://rungameid/4465480//+connect ip:port`（地址仅在点击时拼接，不写入 DOM，沿用 PROJECT §2）；
    AppID 抽成 `CS_APP_ID` 常量便于后续更换；「复制 IP」保留为次按钮。
  - 视觉强化（设计系统升级，PROJECT §2「视觉系统」已同步）：
    - 按钮：Naive Button 覆写强描边/深色字/hover 墨绿字，CSS 常驻微阴影 + hover 浮起；
      主按钮（primary）墨绿填充 + 绿色投影，主/次操作区分明确。
    - 导航当前项改为墨绿实心白字 + 投影。
    - 表格表头改浅绿底 + 墨绿加粗字（theme `DataTable` + 全局 CSS）。
    - 面板/卡片常驻微阴影（`--shadow-card`），`panel-head` 与 NCard 头部加浅绿渐变；
      页面顶部加极浅墨绿晕染。
    - 玩家名（`PlayerCell` / 记录页）改墨绿链接，hover 下划线。
    - 首页 Hero/统计卡、输入框（更强描边 + 聚焦墨绿环）同步。
- 涉及文件：修改 `frontend/src/app/theme.ts`、`frontend/src/styles/global.css`、
  `frontend/src/layouts/DefaultLayout.vue`、`frontend/src/components/PlayerCell.vue`、
  `frontend/src/features/servers/ServersPage.vue`、`frontend/src/features/records/RecordsPage.vue`、
  `PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无（纯前端）。
- 测试与结果（2026-10-18）：
  - `npm run build`（含 `vue-tsc -b`）通过；产物内可见 `steam://rungameid/...` 链接。
  - Playwright 截图核对 1440：服务器默认卡片含「加入游戏 / 复制 IP」、导航实心选中、
    表头浅绿、玩家名墨绿、卡片阴影与渐变均生效；首页/榜单/地图/记录/封禁/档案同步。

## 2026-10-18 服务器列表默认卡片排列 + 按社区分类

- 背景：用户要求服务器列表默认卡片样式，并增加按社区名称分类（CNGOKZ / AXE / HIME 等）。
- 变更内容：
  - 后端新增 `modules/servers/category.rs`（纯函数 + 单测）：`community(name, group_name)` 优先
    `group_name` 并去掉末尾通用词与编号（`AXE GOKZ`→`AXE`、`Node KZ Server`→`Node`）；缺失时从
    名称推导——`【X】`/`[X]` 前缀、常见分隔符（双空格 / ` GOKZ` / ` KZ` / ` |` / ` #` / ` -` / `服`）
    前的前缀，再归一化（`【HIME】GOKZ …`→`HIME`、`KZMANIA GOKZ 2`→`KZMANIA`、
    `UB社区 攀岩竞速 #01 …`→`UB社区`、`Sunl 1服`→`Sunl`、`House of Climb  EU …`→`House of Climb`）。
  - `ServerEntry` 与 `GET /api/v1/servers` 新增 `category` 字段。
  - 前端：服务器页默认视图由列状改为**卡片**；新增社区分类筛选栏（计数倒序、可横向滚动，
    分类消失时自动回到「全部」）；卡片图片左上角与列状视图 meta 均显示社区标签；搜索同时匹配社区名。
- 涉及文件：新增 `backend/src/modules/servers/category.rs`；修改
  `backend/src/modules/servers/{mod,models,repository}.rs`、
  `frontend/src/features/servers/{types.ts,ServersPage.vue}`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无迁移；`GET /api/v1/servers` 响应新增 `category`（additive）。
- 测试与结果（2026-10-18）：
  - `cargo test` **26 passed**（新增 3 个分类单测，覆盖用户给的三组示例与非 `group_name` 场景）、
    `cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`、`npm run build` 全过。
  - 实测 89 台在线服务器归为 20 类：AXE 22 / House of Climb 16 / CNGOKZ 15 / FKZ 6 / UB社区 6 /
    HIME 3（含 1 台 `group_name` 为空、靠 `【HIME】` 推导，与另 2 台合并）/ KZMANIA 3 …
  - Playwright 截图核对：默认卡片、点 CNGOKZ 筛选（在线徽章变 15）、列状视图、600px 分类栏横向滚动。

## 2026-10-18 地图列表改 4 列并显示当前模式 WR（记录人 / 耗时 / 日期）

- 背景：用户反馈"地图列表中三列显示的有点太少了"，要求改 4 列，并在卡片上显示当前地图的
  WR 记录人与记录时间信息。
- 变更内容：
  - 后端：`GET /api/v1/maps` 每张地图新增当前模式主关 WR —— 用 `LEFT JOIN LATERAL`
    （`ORDER BY time_ms LIMIT 1`，命中 `idx_records_map_place`）取记录人与 `created_at`；
    `MapSummary` 增 `wr_player` / `wr_time`（`M:SS.mmm`）/ `wr_date`（`YYYY-MM-DD`）。
    地图详情的 `map.wr_*` 直接用主关榜第一名填充，不重复查询；无成绩的图为 null
    （前端显示"暂无 WR 记录"）。
  - 前端：地图卡片网格宽屏默认 4 列（1100 / 860 / 520 依次降为 3 / 2 / 1 列）；
    卡片底部新增 WR 行（墨绿 WR 徽章 + 记录人 + 绿色等宽耗时，日期固定另起一行保证对齐）。
  - 类型：`features/maps/types.ts` 的 `MapSummary` 同步三个新字段。
- 涉及文件：修改 `backend/src/modules/maps/{models,repository,service}.rs`、
  `frontend/src/features/maps/{types.ts,MapsPage.vue}`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无迁移；`GET /api/v1/maps` 响应新增 `wr_player` / `wr_time` / `wr_date`
  （additive，旧客户端不受影响）。
- 测试与结果（2026-10-18）：
  - `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（23 passed）、
    `npm run build`（含 `vue-tsc -b`）全部通过。
  - 实测 `GET /api/v1/maps?mode=KZT`：如 `bkz_apricity_v3` → WR `Ballistic Bacon 3:05.109 / 2023-07-02`。
  - Playwright 截图核对 1440（4 列）/ 1100（3 列）/ 860（2 列），WR 行与日期对齐正常。

## 2026-10-17 全站视觉优化：面板体系 / 首页 Hero / 详情英雄区 / 名次徽章

- 背景：用户反馈"当前页面有点简约单调了，你来操刀优化一下"。要求保持 PROJECT §2 设计约束
  （白底 + 单一墨绿主色、细边框、克制干净、不做科技感/竞技风），在现有页面结构不变的前提下提升
  信息层次与视觉质感；纯前端改动。
- 变更内容：
  - 设计系统（`styles/global.css` + `app/theme.ts`）：新增 token（`--accent-wash`、
    `--radius-card`、`--shadow-lg`、`--tier-severe`、`--font-mono`）、键盘 `:focus-visible` 描边；
    新增 `.page-head-row`/`.page-head-actions`/`.page-eyebrow`、`.panel/.panel-head/.panel-body`
    面板体系、`.chip`、`.hover-lift`、`.fade-in` 与 `prefers-reduced-motion` 适配；
    表格表头浅底、行 hover、区块标题墨绿竖条；`theme.ts` 补小卡片内边距与 Alert 圆角。
  - 导航/布局：logo 改墨绿方块 KZ 标识（首页可点），Steam 登录按钮加 Steam 图标，
    吸顶栏加细阴影；600px 下 Steam 按钮只留图标，保证链接永不重叠；路由切换加极轻淡入；
    底部栏保持原有紧凑高度。
  - 首页：Hero 增加 eyebrow、两个入口按钮（排行榜/地图）与极浅墨绿晕染；4 个统计卡加线性图标；
    「关于 CNGOKZ」改双栏（正文 + 数据源/覆盖模式/收录内容/统计时间事实列表）；
    NGrid 改响应式 `1 s:2 m:4`（600px 不再挤成 4 列导致数字断行）。
  - 榜单/封禁：表格统一移入 `.panel`（标题 + 说明 + 计数徽章）；封禁理由红色、
    SteamID64 不再换行、服务器列省略号 + tooltip。
  - 地图：卡片图片上叠加当前模式难度徽章，图片 hover 放大，缺图时显示地图名 + 图标占位；
    分页加页码提示。
  - 地图详情：新增英雄区（缩略图 + 三模式 tier / Bonus / 大小 chips + 制作人 / 创意工坊），
    排行榜移入面板。
  - 服务器：头部计数 + 视图切换统一；卡片 hover 浮起、图片 hover 放大；两种视图都加缺图占位；
    列状视图重排（地图名加粗、人数与复制按钮右侧对齐）。
  - 记录：三列模式卡加墨绿圆点标题，时间用等宽绿色强调。
  - 个人资料：新增头像英雄卡（72px 头像 + 名称 + SteamID64 + 区域 chip），两张表移入面板。
  - 登录：Steam 图标 + 文案分栏卡片。
  - 修复（本次发现的存量问题）：
    - DataTable 列 `render` 中 `h()` 创建的类命中不了 scoped CSS（名次徽章、封禁理由、
      榜单文字此前一直没上色）——新增公共组件 `components/RankBadge.vue` 供排行榜/地图详情复用，
      其余改用 `:deep()`。
    - `PlayerCell` 的 SteamID64 在窄列会折行，加 `white-space: nowrap` + 截断。
- 涉及文件：新增 `frontend/src/components/RankBadge.vue`；修改 `frontend/src/styles/global.css`、
  `frontend/src/app/theme.ts`、`frontend/src/layouts/DefaultLayout.vue`、
  `frontend/src/components/PlayerCell.vue`、
  `frontend/src/features/{home,leaderboard,maps,servers,records,bans,profile,auth}/*.vue`、
  `PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无（纯前端展示层）。
- 测试与结果（2026-10-17）：
  - `npm run build`（含 `vue-tsc -b`）通过。
  - 用 Playwright/Chromium 截图核对 1440（首页/榜单/地图/地图详情/服务器两种视图/记录/封禁/档案/登录）、
    1100、860、600 断点：导航不换行不重叠；600px 统计卡 1 列；前三名金银铜徽章、
    封禁理由红色、缺图占位、hover 与工具条均正常。
  - 实测数据：首页 1,757 玩家 / 1,044 地图 / 89 在线服务器 / 163,352 记录；
    地图详情前 100 名与制作人（Badges）正常。
  - 截图环境说明：验证机无中文字体，已在本机用户目录安装 Noto Sans CJK 供截图核对（与仓库无关）。

## 2026-10-17 首页公告 / 地图卡片 / 服务器双视图 / 记录头像

- 背景：用户反馈"还有部分没有恢复"，并明确四项：① 恢复首页「最新公告」的内容与位置；
  ② 地图页仿 gokz.top 改卡片排列，显示地图图片、名称、制作人、难度、是否有 Bonus；
  ③ 服务器页提供卡片排列与列状排列两种视图，并显示地图图片等；④ 最新记录显示玩家头像并重排布局，
  避免文字换行。图片先接入上游公开地图图库（后续按要求可换自建图床）。
- 变更内容：
  - 后端数据：
    - 迁移 `0004_map_meta.sql`：`maps.mappers TEXT[]`、`servers.map_tier`、`servers.player_names TEXT[]`。
    - 新增数据源 `sources/kzgo.rs`：kzgo 地图元数据里的**制作人显示名**（gokz.top 只给作者 SteamID）；
      `run_maps` 在地图库写完后补写，实测 **963/1044** 张有制作人名。
      **注意上游 `mapperNames` 可能含 null**（实测 `kz_south` 为 `["nopey", null]`），已容错。
    - `run_servers` 写入当前地图难度与实时在线玩家昵称；服务器读取时用 `maps` 关联难度，
      避免同步先后导致 tier 为 0。**注意上游 `live_status` 的数值/布尔/数组字段都可能是 null**，
      已全部改为 `Option`（此前正是它让 `servers` 任务长期 failed）。
    - `GET /api/v1/records` 增加 `steam_id64`（前端生成头像与跳转资料页用）。
    - 同步队列改进：**全局任务优先于逐玩家的档案补全**，否则玩家任务会把全局同步饿死；
      定时入队顺序改为"地图 → 榜单 → 服务器 → 封禁 → 增量"。
    - 运维工具 `ingest-key` 增加 `jobs`（查看队列与失败原因）与 `requeue <kind> <key>`（立即重跑）。
  - 前端：
    - 首页：恢复「最新公告」区块（位置在「关于 CNGOKZ」之后，三条公告内容同原原型），
      卡片样式沿用原型的 `.announcement-*`。
    - 地图页：改为卡片网格（地图图片 / 名称 / 难度徽章 / 制作人 / Bonus 标记），
      保留搜索 + 难度筛选，新增分页（每页 24 张）。
    - 服务器页：新增视图切换（卡片排列 / 列状排列，**默认列状**）；两种视图都显示地图图片、
      当前地图、难度与区域、人数、在线玩家标签；地址仍只经「复制 IP」获取（不渲染明文）。
    - 最新记录：新增玩家头像列（头像 + 名称可点进资料页），列宽固定 + 省略号（hover 显示全称），
      不再出现文字换行。
    - 新增 `utils/mapImage.ts`（地图图片地址统一出口）与 `features/home/announcements.ts`（公告静态数据）。
- 涉及文件：新增 `backend/migrations/0004_map_meta.sql`、`backend/src/sources/kzgo.rs`、
  `frontend/src/utils/mapImage.ts`、`frontend/src/features/home/announcements.ts`；
  修改 `backend/src/sources/{mod.rs,gokz_top.rs}`、
  `backend/src/modules/{sync/{jobs,queue,worker,repository}.rs,maps/*,servers/*,records/*}`、
  `backend/src/bin/ingest-key.rs`、`frontend/src/features/{home,maps,servers,records}/*`、
  `PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：`0004` 已应用；`GET /api/v1/maps` 增加 `mappers`；
  `GET /api/v1/servers` 增加 `map_tier`、`player_names`；`GET /api/v1/records` 增加 `steam_id64`。
- 测试与结果（2026-10-17）：
  - `cargo test` **23 passed**（新增：kzgo 含 null 制作人的解析、服务器 `live_status` 全 null 的解析）；
    `cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`、`npm run build`（含 `vue-tsc`）全过。
  - 实测数据：制作人 **963/1044**；服务器 89 台在线、其中 **42 台带在线玩家昵称**；
    当前地图难度按 `maps` 关联正确（如 CNGOKZ 9 服 `kz_hb_lowlita` → Tier 4）。
  - 页面：`/`、`/maps`、`/maps/11`、`/servers`、`/records`、`/bans`、`/leaderboard`、`/profile`、`/login` 全 200。
  - 修过的问题：kzgo `mapperNames` 含 null 导致整批解析失败；`live_status` 字段为 null 导致
    `servers` 任务一直 failed（表现为"服务器没有玩家/难度"）；玩家补全任务抢占全局同步。

## 2026-10-17 布局回归：恢复原有页面骨架与设计，仅以真实数据填充

- 背景：用户要求"改成昨天那种布局，网站的布局与样式和设计都不要改变"。此前接真实数据时，
  首页/地图/服务器/个人资料除了填数据，还顺带改了布局（6 卡统计 + 两张附加表、自定义服务器
  卡片列表、自定义档案网格、导航多一项、页脚多一行），属于超出要求的改动，本次全部回归。
- 变更内容：
  - 首页：回到「`page-head` + 4 列 `NGrid`（4 个 `NStatistic`）+ 「关于 CNGOKZ」卡」的原结构，
    删除附加的"最新记录/最活跃服务器"两张表；统计数字改为真实值（取不到时显示 `—`）。
  - 地图 / 服务器 / 个人资料：统一回到「`page-head` + 单张 `NCard`」骨架，真实内容
    （筛选 + `NDataTable`）放在卡片内部；删除自定义的服务器卡片网格与档案网格，
    只用站内已有的 `NDataTable` / `.toolbar` / `NTag` 组合。
  - 地图详情（新页面，昨天不存在）：同样只用 `page-head` + `NCard` + `NDataTable`。
  - 导航：去掉新增的「个人资料」项，恢复昨天的 6 项（资料页仍可从任意列表点击玩家名进入，
    或直接访问 `/profile/<SteamID64>`）。
  - 页脚：恢复昨天的三个链接与版权行，去掉新增的"数据来源"一行；数据来源标注保留在首页
    「关于 CNGOKZ」卡片内，仍满足 PROJECT §9.9 的标注要求。
  - 头像：沿用上一轮恢复的按 SteamID64 生成的 DiceBear 图（与重构前一致）。
- 涉及文件：修改 `frontend/src/layouts/DefaultLayout.vue`、`frontend/src/features/home/HomePage.vue`、
  `frontend/src/features/maps/MapsPage.vue`、`frontend/src/features/maps/MapDetailPage.vue`、
  `frontend/src/features/servers/ServersPage.vue`、`frontend/src/features/profile/ProfilePage.vue`、
  `CHANGELOG.md`。
- 数据库与 API 变化：无（纯前端展示层）。
- 测试与结果（2026-10-17）：
  - `npm run build`（含 `vue-tsc -b`）通过，产物哈希更新为 `assets/index-BlPw28kB.js`；
    页面 `/`、`/maps`、`/maps/11`、`/servers`、`/profile`、`/records`、`/bans`、`/leaderboard`、
    `/login` 全部 200（普通刷新 + 必要时强刷即可生效）。
  - 与昨天的对比：`frontend/src/layouts/DefaultLayout.vue`、`frontend/src/styles/global.css`、
    `frontend/src/app/theme.ts`、`frontend/src/app/App.vue`、`frontend/index.html` 的 `git diff`
    **均为空**（与重构前完全一致）；首页结构核对为 `NGrid :cols="4"` + 4 个 `NStatistic`
    + 「关于 CNGOKZ」卡。
  - 剩余差异仅为"页面内容"（真实数据的表格）以及新增的 api/types/`PlayerCell`/`avatar` 工具文件。

## 2026-10-17 外观回归：恢复页脚链接与头像生成图

- 背景：用户反馈"网站样式跟昨天不同"。排查结论：设计系统（`global.css`、`styles/global.css` 与
  `app/theme.ts`、`App.vue`、`index.html`）**今天完全没改**；变化来自"占位页换成真实数据页"，
  外加两处与"接真实数据"无关的顺带改动（页脚链接、头像），后者属于不该动的部分，本次还原。
- 变更内容：
  - 页脚：恢复「关于我们 / 规则说明 / API 文档」三个链接（`.footer-links` 样式一并恢复），
    保留一行小字「数据来源：GOKZ.TOP / KZTimer Global API」。
  - 头像：新增 `frontend/src/utils/avatar.ts`（`avatarUrl` / `fallbackAvatar`），
    没有真实头像时按 SteamID64 生成与重构前一致的 DiceBear identicon，
    不再是灰色首字母圆圈；`PlayerCell.vue` 与 `ProfilePage.vue` 统一走该工具函数。
- 涉及文件：新增 `frontend/src/utils/avatar.ts`；修改
  `frontend/src/layouts/DefaultLayout.vue`、`frontend/src/components/PlayerCell.vue`、
  `frontend/src/features/profile/ProfilePage.vue`、`CHANGELOG.md`。
- 数据库与 API 变化：无（纯前端展示）。
- 测试与结果（2026-10-17）：
  - `npm run build`（含 `vue-tsc -b`）通过，产物哈希更新为 `assets/index-CIJUBwmL.js`，
    站点 `/` 返回 200 且 `index.html` 已指向新产物（普通刷新即可生效）。
  - 未改动主题与全局样式：`git diff` 对 `frontend/src/styles/global.css`、`frontend/src/app/theme.ts`、
    `frontend/src/app/App.vue`、`frontend/index.html` 均为空。

## 2026-10-17 收尾：全站真实数据验收与文档一致性

- 背景：目标（真实数据替代全部模拟数据 + §9 服务端落地）完成后做最终验收：逐页逐接口核对、
  清掉最后残留的模拟痕迹、修正文档里过期的"尚未实现"表述。
- 变更内容：
  - 前端登录页：删除 `mockLogin()` 假装登录的模拟动作，改为明确提示「Steam OpenID 尚未接入」
    并说明免登录可用范围（排行榜 / 地图 / 服务器 / 记录 / 封禁 / 按 SteamID64 查资料）。
  - `PROJECT.md`：§4 已落地模块补上 `ingest` 与运维 bin；§5 的
    `server_groups` / `server_keys` 说明由"接口尚未实现"改为"接收端已使用"；
    §9 引言由"规划项中同步标注"改为"已标注落地状态"。
  - `CHANGELOG.md`：同日条目按「背景 / 变更内容 / 涉及文件 / 数据库与 API 变化 / 测试与结果」
    统一整理与措辞校正。
- 涉及文件：修改 `frontend/src/features/auth/LoginPage.vue`、`PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无（纯前端文案与文档）。
- 测试与结果（2026-10-17，`PORT=3000`，连 62 测试库）：
  - 页面全部 200：`/`、`/maps`、`/maps/11`、`/servers`、`/profile`、
    `/profile/76561198297415967`、`/records`、`/bans`、`/leaderboard`、`/login`。
  - 接口全部 200：`/health/ready`、`/stats`、`/leaderboard?mode=KZT`、`/records`、
    `/bans`、`/maps?tier=5&mode=KZT`、`/maps/11?mode=KZT`、`/servers`、
    `/players/76561198297415967`。
  - 运行数据快照（截至本次验收）：1708 玩家 / 1044 地图 / 162,091 成绩 / 89 台在线服务器 /
    500 条封禁。
  - 质量门：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
    `cargo test`（21 passed）、`npm run build`（含 `vue-tsc`）全部通过；
    `backend/src` 与 `frontend/src` 中已无 `mock` 引用。

## 2026-10-17 上报接收端（ingest）落地：原生批量 + GlobalAPI 兼容层

- 背景：§9.6 的接收端是插件接入的前提，属服务端工作，先把这一侧做完（插件仍后置）。
- 变更内容：
  - 结构：新增 `backend/src/lib.rs`，把各模块暴露给 `main.rs`、`src/bin/*` 与后续集成测试；
    `main.rs` 改为使用库目标（`cngokz_backend::…`）。
  - 接收端 `modules/ingest`（dto / models / service / repository / handlers / routes）：
    - `POST /api/v1/ingest/records`：批量上报，逐条返回 `accepted | duplicate | rejected`
      （含 `place` 与拒绝原因），单批上限 200 条。
    - `POST /api/v2.0/records`：GlobalAPI 兼容层——请求体 `{steam_id, map_id, mode, stage,
      tickrate, teleports, time}`（`time` 为秒、`mode` 为 200/201/202），
      返回 `{place, top_place, top_overall_place}`，HTTP 201。
  - 鉴权：`X-ApiKey` → `server_keys`（`revoked_at IS NULL`）→ 服务器组；缺失/无效返回 401。
  - 幂等：原生用客户端 `record_uuid`（`records.external_id` 唯一键）；兼容层官方客户端没有
    幂等键，改用自然键 + **10 分钟时间窗**判定重试，重复提交不写入新行。
  - 计分：受理后按本站口径（`(地图, 关卡, 传送口径)` 取最佳求和）重算受影响玩家的
    `player_scores.points_local`；**单条成绩分（GOKZ 公式）仍未落地**，入库 `points = 0`、
    响应 `points = null`（PROJECT §9.4）。
  - 工具：新增 `backend/src/bin/ingest-key.rs`：`create-group` / `issue-key` / `list` /
    `revoke` / `delete-record`（key 只显示一次，list 只显示前缀）。
  - `AppError` 补 `Display` 实现（便于按条错误转换），统一错误信封不变。
- 涉及文件：新增 `backend/src/lib.rs`、`backend/src/modules/ingest/*`、
  `backend/src/bin/ingest-key.rs`；修改 `backend/src/main.rs`、`backend/src/app.rs`、
  `backend/src/modules/mod.rs`、`backend/src/errors.rs`、`backend/Cargo.toml`、
  `PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无新迁移（`server_groups` / `server_keys` 已在 `0002`）；
  新增两个 POST 接口（见上），其余接口不变。
- 测试与结果（2026-10-17）：
  - `cargo test` 21 passed（新增 SteamID32→64、模式归一化、秒→毫秒 单测）；
    `cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`、`npm run build` 通过。
  - 端到端（端口 3000，连 62 测试库，实测后已清理测试数据）：
    无 key / 坏 key → 401；原生批量 1 合法 + 1 非法 →
    `accepted(place=1)` + `rejected("steam_id64 应为 17 位数字")`；
    同一条再次上报 → `duplicate`；兼容层 → `201 {place:1,top_place:1,top_overall_place:1}`，
    同成绩二次提交经玩家档案核对**只有 1 行**（窗口去重生效）。
  - 修过的问题：`sum(bigint)` 在 PostgreSQL 返回 `NUMERIC`，按 `i64` 解码失败导致 500
    （改为 `::BIGINT`）。

## 2026-10-17 真实数据接入落地：删除全部模拟数据（网站先行，插件后置）

- 背景：按 §9 方案开始实现。目标是"网站先用真实数据跑起来"，插件端后置。
- 变更内容：
  - 数据库：新增 `0002_external_data.sql` 并在 62 测试库应用——`maps` 三模式 tier / 工坊 /
    作者 / bonus、`players.region`、`records` provenance（source/external_id）与
    stage/tickrate/teleports/points_local、`servers` 在线状态、`bans` 改自增 id（可多条），
    新增 `player_scores` / `sync_jobs` / `server_groups` / `server_keys`。
  - 后端新增 `src/sources/`：GOKZ.TOP 客户端（可识别 UA、单源 400ms 限速、三次退避重试、
    外部 DTO 不外泄）。**注意：上游对默认 UA 返回 403，必须带 UA。**
  - 后端新增 `modules/sync/`：`sync_jobs` 队列（幂等入队、`FOR UPDATE SKIP LOCKED` 认领、
    指数退避、8 次后置 `dead`）+ 后台 worker（启动种子、recent 5min / servers 10min /
    bans 1h / leaderboard 与 maps 24h 定时、最多 3 个任务并发、重启接管 running 任务）+
    任务实现（maps / leaderboard / recent / map 整图 / player 档案 / servers / bans），
    全部幂等 upsert；地图入库用 `jsonb_to_recordset` 单次往返。
  - 后端新增 `modules/scoring/`：本站汇总口径（同一 (地图, 关卡, 是否用传送) 取最佳后求和 + 平均分）。
  - 后端新增读模块 `maps`（列表过滤 + 详情前 100 + `syncing` 标记）、`servers`、`players`
    （首次访问触发后台同步）、`stats`；`leaderboard` / `records` / `bans` 由模拟层改读本站 PG，
    **删除 `backend/src/modules/mock.rs`**。
  - 前端：`home` / `maps`（新增 `/maps/:id` 详情）/ `servers` / `profile`
    （新增 `/profile/:steam_id?`）四页接真实接口；新增公共组件 `components/PlayerCell.vue`
    （榜单/封禁/地图详情/首页 4 处使用者，无头像时用昵称首字占位）；导航新增「个人资料」；
    页脚标注数据来源；移除全部占位与模拟文案（登录页改为明确提示「Steam OpenID 尚未接入」，
    不再有假装登录的模拟动作）；榜单 `rank` 改为**本站名次**、最近游玩/记录数在未同步时显示"—"。
- 涉及文件：新增 `backend/src/sources/*`、
  `backend/src/modules/{sync,scoring,maps,servers,players,stats}/*`、
  `backend/migrations/0002_external_data.sql`、`frontend/src/components/PlayerCell.vue`、
  `frontend/src/features/{maps,servers,profile,home}/*`；
  修改 `backend/src/{main,state,app,modules/mod}.rs`、
  `backend/src/modules/{leaderboard,records,bans}/*`、`frontend/src/{router,layouts,utils}/*`、
  `frontend/src/features/{leaderboard,bans}/*`、`PROJECT.md`、`CHANGELOG.md`；
  删除 `backend/src/modules/mock.rs`。
- 数据库与 API 变化：
  - DB：`0002` 与 `0003_scoring_fix.sql` 均已应用（`0003` 清空旧 `points_local` 并把已同步的
    玩家任务重新排队，按 TP/PRO 分开的正确口径重算）。
  - API：新增 5 组——`GET /api/v1/maps`、`GET /api/v1/maps/{id}`、`GET /api/v1/servers`、
    `GET /api/v1/players/{steam_id64}`、`GET /api/v1/stats`；`/leaderboard` 的 `rank` 改为本站名次、
    `last_played` / `last_played_days_ago` 可为 `null`；`/records` 的 `points` 为整数。
    `ingest` 上报接收端当时**仍未实现**（§9.6），随后在同日上一条「上报接收端落地」中完成。
- 测试与结果（2026-10-17）：
  - `cargo test` 16 passed；`cargo clippy --all-targets -- -D warnings` 通过；
    `npm run build`（含 `vue-tsc -b`）通过。
  - 运行时（`PORT=18091`，连 62 测试库，浏览器/接口直连）：`/health/ready` PG/Redis 双 true；
    实测导入 1000+ 张地图、1100+ 玩家、8 万+ 成绩、81 台在线服务器、500 条封禁；
    逐个核对榜单（200 行真实玩家/积分/完成图数）、地图 tier 过滤（T5+KZT 70 张）、
    地图详情前 100、服务器（含 CNGOKZ 服实时地图与人数）、玩家档案（三模式排名/积分）、
    首页统计、封禁搜索分页。
  - 修过的问题：上游 `leaderboard?limit` 上限 100（改自动分页）；上游对默认 UA 返回 403；
    进程重启后 `running` 任务无人接管（启动时重置为 `pending`）；地图详情曾因"已导入部分成绩"
    误判同步完成（改为按 `sync_jobs` 的 done 状态判断）；SPA 深链刷新返回 404
    （`ServeDir::not_found_service` 会强制 404，改用 `fallback` 返回 200 的 index.html，
    同时给 API 加 JSON 404 兜底）；本地汇总把 TP/PRO 合成一张榜导致少算（改为按
    `(地图, 关卡, teleports > 0)` 分组）；`points_local` 只有部分成绩时偏小，
    若用于榜单会得出错误排名（**榜单改用上游积分口径**，见 PROJECT §9.4）。

## 2026-10-17 外部数据接入与独立计分方案定稿（仅文档）

- 背景：本站全部页面仍走模拟数据；需要"不从零计分"——把外部站点已有的成绩接进来，
  同时保证本站**独立计分**、上传到多个站点**互不影响**；服务器插件端尚未开发，
  故本轮只定方案与规范。为此先做了实测调研与源码核实（结论均已写入 PROJECT §9）：
  - 生态：服务器侧是 GOKZ + GlobalAPI 客户端上报；网站只有"只读型"（kzgo.eu 读
    `kztimerglobal.com`）与"导入型"（gokz.top 的 `globalapiRecordsSync` 从 GlobalAPI 导入）。
    **未接入上报的服务器，任何站点都拿不到其成绩**。
  - 核实 `GlobalAPI-SMPlugin`（Bitbucket）：只有 `GlobalAPI_Debug` / `GlobalAPI_Staging`
    两个 convar，base URL 为编译期常量（生产 kztimerglobal / 预发 staging），
    **不支持自定义 URL**；上报为 `POST {base}/records` + `X-ApiKey`，body 为
    `{steam_id,map_id,mode,stage,tickrate,teleports,time}`。→ "服务器只改配置就推到我们站"不可行。
  - `gokz-top-plugins`（kzcharm）是游戏内读取 + 服务器遥测插件，**不含成绩上传**；
    成绩上传仍由 GOKZ 的 GlobalAPI 完成。
  - `kztimerglobal` 的 `player_ranks`（现成积分榜）服务端 500、超时，不可依赖 → 必须自建计分。
  - 实测 `api.gokz.top/v1/*` 匿名可读且字段最全：玩家榜（`count` 约 10.6 万）、
    地图（含 KZT/SKZ/VNL 三模式 tier、bonus 数）、记录（可按 steamid64/map_name/stage 过滤）、
    bans、servers、jumpstats、records/pb。
- 变更内容（仅文档，未改代码）：
  - `PROJECT.md` 新增 §9「外部数据接入与独立计分（GlobalAPI 生态）」：生态机制与红线、
    数据源优先级表与关键接口、三条数据通路（导入 / 原生上报 / GlobalAPI 兼容上报）、
    独立计分与 `points_local`/`points_source` 双分与上游 oracle 单测、按需同步策略
    （玩家懒加载 / 地图整图同步 / 榜单按页 / recent 增量 + `sync_jobs` 队列）、
    两套接收端契约（幂等键与 `X-ApiKey`）、`cngokzapi` 插件约定（瘦上报器、独立 key 文件、
    本地队列退避、多站独立 worker）、三插件共存冲突规避 8 条、合规与待确认项；
    原 §9 顺移为 §10 并更新当前状态与下一步。
  - 同步补充：§1 目标（数据来源与独立计分）、§3.1 选型表（数据接入/计分两行）、
    §3.3 后端分层（新增 `sources/`，`ingest`/`scoring`/`sync`，外部 HTTP 边界）、
    §4 规划目录、§5 规划表结构与幂等键、§6 规划接口、§7 外部请求规范、§8 流程。
- 涉及文件：修改 `PROJECT.md`、`CHANGELOG.md`。
- 数据库与 API 变化：无（规划项已在 §5/§6 标注为"落地前不存在"，随 `0002` 迁移与代码落地）。
- 测试与结果（2026-10-17）：本轮为文档变更，未改代码；无 PG/Redis 依赖。
  文档自检：章节编号连续（§9 新增、原 §9→§10），外部交叉引用 `§3.3`/`§5` 未被破坏；
  上游接口结论均来自实测（`api.gokz.top`、`kztimerglobal.com`）与源码（GlobalAPI-SMPlugin、
  gokz、gokz-top-plugins）。

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
