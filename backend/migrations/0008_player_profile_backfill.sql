-- 真实数据补全：为所有缺少头像或国籍的玩家入队轻量档案同步。
-- 这些玩家来自排行榜 / 地图榜 / 最近记录（此前只写入昵称），头像会退化为 DiceBear 占位图。
-- 每个任务只请求 GOKZ.TOP `GET /players/{steamid64}` 一次，按限速在后台慢慢跑完。

INSERT INTO sync_jobs (kind, key, status, attempts, next_run_at, updated_at)
SELECT 'player_profile', steam_id::text, 'pending', 0, now(), now()
FROM players
WHERE avatar_url IS NULL OR avatar_url = '' OR country IS NULL
ON CONFLICT (kind, key) DO UPDATE SET
    status      = 'pending',
    attempts    = 0,
    next_run_at = now(),
    last_error  = NULL,
    updated_at  = now();
