-- 玩家「全部已完成地图」需要完整的 PB 覆盖：旧同步只按 `/records?steamid64=&limit=1000`
-- 拉最近成绩，顶级玩家地图完成数会被截断（如 955 张只剩 150 张）。
-- 新同步会按模式分页拉 `/records/pb`（NUB/PRO 主关），这里把 player 任务重置以回填。

UPDATE sync_jobs
SET status = 'pending', attempts = 0, next_run_at = now(), updated_at = now()
WHERE kind = 'player';
