-- 排行榜全量导入：上游实际人数 KZT≈10.3 万 / SKZ≈9.4 千 / VNL≈3.4 千，
-- 之前只导入每个模式前 200。这里为三模式各入队一次全量回扫（key 后缀 :full）。
-- 之后由 worker 每 6 小时自动全量刷新，10 分钟刷新前 1000 名。

INSERT INTO sync_jobs (kind, key, status, attempts, next_run_at, updated_at)
VALUES
    ('leaderboard', 'KZT:full', 'pending', 0, now(), now()),
    ('leaderboard', 'SKZ:full', 'pending', 0, now(), now()),
    ('leaderboard', 'VNL:full', 'pending', 0, now(), now())
ON CONFLICT (kind, key) DO UPDATE SET
    status      = 'pending',
    attempts    = 0,
    next_run_at = now(),
    last_error  = NULL,
    updated_at  = now();
