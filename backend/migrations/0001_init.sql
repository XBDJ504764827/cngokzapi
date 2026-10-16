-- CNGOKZ 初始表结构（与 PROJECT.md §5 一致，另含 bans 封禁表）。
-- 应用：sqlx migrate run（需先起 PostgreSQL，见 .env.example）。

CREATE TABLE IF NOT EXISTS players (
    steam_id    BIGINT PRIMARY KEY,
    name        TEXT NOT NULL,
    avatar_url  TEXT NOT NULL DEFAULT '',
    points      BIGINT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    banned_until TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS maps (
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    tier        SMALLINT NOT NULL CHECK (tier BETWEEN 1 AND 8),
    mode        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS records (
    id               BIGSERIAL PRIMARY KEY,
    player_steam_id  BIGINT NOT NULL REFERENCES players (steam_id),
    map_id           BIGINT NOT NULL REFERENCES maps (id),
    mode             TEXT NOT NULL,
    time_ms          INTEGER NOT NULL,
    points           INTEGER NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_records_map_mode_time
    ON records (map_id, mode, time_ms);
CREATE INDEX IF NOT EXISTS idx_records_player_created
    ON records (player_steam_id, created_at DESC);

CREATE TABLE IF NOT EXISTS servers (
    id           BIGSERIAL PRIMARY KEY,
    name         TEXT NOT NULL,
    map          TEXT NOT NULL DEFAULT '',
    ip           TEXT NOT NULL,
    port         INTEGER NOT NULL,
    player_count INTEGER NOT NULL DEFAULT 0,
    max_players  INTEGER NOT NULL DEFAULT 0,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS bans (
    steam_id   BIGINT PRIMARY KEY REFERENCES players (steam_id),
    name       TEXT NOT NULL,
    avatar_url TEXT NOT NULL DEFAULT '',
    reason     TEXT NOT NULL,
    server     TEXT NOT NULL DEFAULT '',
    detail     TEXT NOT NULL DEFAULT '',
    banned_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_bans_banned_at ON bans (banned_at DESC);

CREATE TABLE IF NOT EXISTS admins (
    steam_id   BIGINT PRIMARY KEY,
    role       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
