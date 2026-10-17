-- 外部数据接入（PROJECT §9）：为导入 / 上报 / 计分补齐字段与表。
-- 幂等：全部 IF NOT EXISTS / IF EXISTS，可重复执行。
-- 应用：DATABASE_URL=... sqlx migrate run --source backend/migrations

-- ========== maps：按模式 tier + 工坊与作者等元数据 ==========
ALTER TABLE maps ALTER COLUMN mode SET DEFAULT 'OVR';
-- tier 允许 0（该模式未评级）
ALTER TABLE maps DROP CONSTRAINT IF EXISTS maps_tier_check;
ALTER TABLE maps ADD CONSTRAINT maps_tier_check CHECK (tier BETWEEN 0 AND 8);
ALTER TABLE maps
    ADD COLUMN IF NOT EXISTS external_id BIGINT,
    ADD COLUMN IF NOT EXISTS tier_kzt    SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS tier_skz    SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS tier_vnl    SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS bonuses     SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS workshop_id BIGINT,
    ADD COLUMN IF NOT EXISTS filesize    BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS validated   BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS authors     TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS synced_at   TIMESTAMPTZ;
CREATE UNIQUE INDEX IF NOT EXISTS uq_maps_external ON maps (external_id);

-- ========== players：外部档案字段 ==========
ALTER TABLE players
    ADD COLUMN IF NOT EXISTS region           TEXT,
    ADD COLUMN IF NOT EXISTS avatar_synced_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS updated_at       TIMESTAMPTZ NOT NULL DEFAULT now();

-- ========== records：来源标记（provenance）与计分 ==========
ALTER TABLE records
    ADD COLUMN IF NOT EXISTS source           TEXT NOT NULL DEFAULT 'gokz_top',
    ADD COLUMN IF NOT EXISTS external_id      TEXT,
    ADD COLUMN IF NOT EXISTS stage            INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS tickrate         INTEGER NOT NULL DEFAULT 128,
    ADD COLUMN IF NOT EXISTS teleports        INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS points_local     INTEGER,
    ADD COLUMN IF NOT EXISTS origin_server_id BIGINT,
    ADD COLUMN IF NOT EXISTS server_name      TEXT,
    ADD COLUMN IF NOT EXISTS server_group     TEXT,
    ADD COLUMN IF NOT EXISTS wr_time_ms       INTEGER,
    ADD COLUMN IF NOT EXISTS is_valid         BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS updated_at       TIMESTAMPTZ NOT NULL DEFAULT now();
-- external_id 幂等（同一上游记录只落一次）
CREATE UNIQUE INDEX IF NOT EXISTS uq_records_external ON records (external_id);
-- 自然键幂等（插件上报 / 导入撞车时去重）
CREATE UNIQUE INDEX IF NOT EXISTS uq_records_natural
    ON records (player_steam_id, map_id, mode, stage, tickrate, teleports, time_ms, created_at);
CREATE INDEX IF NOT EXISTS idx_records_created ON records (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_records_map_place ON records (map_id, mode, stage, time_ms);

-- ========== servers：外部标识与在线状态 ==========
ALTER TABLE servers
    ADD COLUMN IF NOT EXISTS external_id     TEXT,
    ADD COLUMN IF NOT EXISTS country         TEXT,
    ADD COLUMN IF NOT EXISTS city            TEXT,
    ADD COLUMN IF NOT EXISTS region          TEXT,
    ADD COLUMN IF NOT EXISTS group_name      TEXT,
    ADD COLUMN IF NOT EXISTS group_custom_id TEXT,
    ADD COLUMN IF NOT EXISTS is_public       BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS online          BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS last_seen_at    TIMESTAMPTZ;
CREATE UNIQUE INDEX IF NOT EXISTS uq_servers_external ON servers (external_id);

-- ========== bans：外部封禁字段 ==========
-- 同一玩家可能有多条封禁（不同服务器/时间），主键改为自增 id，steam_id 不再唯一。
ALTER TABLE bans DROP CONSTRAINT IF EXISTS bans_pkey;
ALTER TABLE bans ADD COLUMN IF NOT EXISTS id BIGSERIAL PRIMARY KEY;
ALTER TABLE bans
    ADD COLUMN IF NOT EXISTS external_id TEXT,
    ADD COLUMN IF NOT EXISTS ban_type    TEXT,
    ADD COLUMN IF NOT EXISTS stats       TEXT,
    ADD COLUMN IF NOT EXISTS expires_at  TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS source      TEXT NOT NULL DEFAULT 'gokz_top';
CREATE UNIQUE INDEX IF NOT EXISTS uq_bans_external ON bans (external_id);
CREATE INDEX IF NOT EXISTS idx_bans_steam ON bans (steam_id);

-- ========== player_scores：榜单汇总（上游口径 + 本站口径） ==========
CREATE TABLE IF NOT EXISTS player_scores (
    player_steam_id  BIGINT NOT NULL REFERENCES players (steam_id) ON DELETE CASCADE,
    mode             TEXT NOT NULL,
    points           BIGINT NOT NULL DEFAULT 0,
    points_local     BIGINT,
    rank             INTEGER,
    rating           DOUBLE PRECISION,
    maps_completed   INTEGER NOT NULL DEFAULT 0,
    wrs_nub          INTEGER NOT NULL DEFAULT 0,
    wrs_pro          INTEGER NOT NULL DEFAULT 0,
    records_900_plus INTEGER NOT NULL DEFAULT 0,
    records_800_plus INTEGER NOT NULL DEFAULT 0,
    last_played      TIMESTAMPTZ,
    source           TEXT NOT NULL DEFAULT 'gokz_top',
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (player_steam_id, mode)
);
CREATE INDEX IF NOT EXISTS idx_player_scores_mode_points ON player_scores (mode, points DESC);
CREATE INDEX IF NOT EXISTS idx_player_scores_mode_rank ON player_scores (mode, rank);

-- ========== sync_jobs：按需回源队列（PROJECT §9.5） ==========
CREATE TABLE IF NOT EXISTS sync_jobs (
    id          BIGSERIAL PRIMARY KEY,
    kind        TEXT NOT NULL,
    key         TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending',
    attempts    INTEGER NOT NULL DEFAULT 0,
    next_run_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_error  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (kind, key)
);
CREATE INDEX IF NOT EXISTS idx_sync_jobs_pending ON sync_jobs (status, next_run_at);

-- ========== 服务器组与上报 key（ingest 鉴权，PROJECT §9.6） ==========
CREATE TABLE IF NOT EXISTS server_groups (
    id         BIGSERIAL PRIMARY KEY,
    name       TEXT NOT NULL,
    custom_id  TEXT UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS server_keys (
    id         BIGSERIAL PRIMARY KEY,
    group_id   BIGINT NOT NULL REFERENCES server_groups (id) ON DELETE CASCADE,
    api_key    TEXT NOT NULL UNIQUE,
    label      TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ
);
