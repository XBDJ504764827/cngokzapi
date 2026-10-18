-- ========== servers：来源标记与 Steam / 插件合并（PROJECT §9.2 / §9.10） ==========
-- 服务器列表改为两个来源：
--   1) 装了本站插件的服务器通过 `POST /api/v1/ingest/servers` 上报（source='plugin'，优先）；
--   2) Steam Web API `IGameServersService/GetServerList` 拉取在线 KZ 服（source='steam'）。
-- 两者以 (ip, port) 合并，同一服务器插件数据优先（见 modules/sync/repository.rs）。
ALTER TABLE servers
    ADD COLUMN IF NOT EXISTS source      TEXT NOT NULL DEFAULT 'steam',
    ADD COLUMN IF NOT EXISTS appid       BIGINT,
    ADD COLUMN IF NOT EXISTS game_mode   TEXT,
    ADD COLUMN IF NOT EXISTS steam_id    TEXT,
    ADD COLUMN IF NOT EXISTS reported_at TIMESTAMPTZ;

CREATE UNIQUE INDEX IF NOT EXISTS uq_servers_addr ON servers (ip, port);
CREATE INDEX IF NOT EXISTS idx_servers_source ON servers (source);
