-- 实时增量同步游标（Phase 1-4）：
-- 1) KZTimer recent 按 record id 游标，避免重复处理；
-- 2) gokz.top records/bans 按 updated_since 游标增量拉取（含软删除/封禁变更）。
CREATE TABLE IF NOT EXISTS sync_state (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 首次只回看最近窗口，避免把全量历史再拉一遍。
INSERT INTO sync_state (key, value)
VALUES
    ('kztimer_recent_id', '0'),
    (
        'gokz_records_updated',
        to_char(
            (now() AT TIME ZONE 'UTC') - interval '10 minutes',
            'YYYY-MM-DD"T"HH24:MI:SS"Z"'
        )
    ),
    (
        'gokz_bans_updated',
        to_char(
            (now() AT TIME ZONE 'UTC') - interval '1 hour',
            'YYYY-MM-DD"T"HH24:MI:SS"Z"'
        )
    )
ON CONFLICT (key) DO NOTHING;
