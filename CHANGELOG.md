# 开发日志（CHANGELOG）

> 记录已经发生的事情：每次开发修改了什么、为什么修改、解决了什么问题、
> 修改了哪些文件、数据库和 API 有什么变化，以及什么时候完成。

格式约定：

- 按日期倒序追加，标题为 `## YYYY-MM-DD 主题`。
- 每条写清：背景 / 变更内容 / 涉及文件 / 数据库与 API 变化 / 测试与结果。

---

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
