-- 计分口径修正（见 PROJECT §9.4）：
-- 1) GOKZ 的 TP / PRO 是两张独立榜单、各自给分，本地汇总必须按
--    (地图, 关卡, teleports > 0) 分组取最佳；旧数据按 (地图, 关卡) 分组，会少算。
-- 2) 旧的 points_local 同时受「只同步了部分成绩」影响，不能直接沿用：
--    清空后由玩家同步任务按新口径重算（榜单不使用 points_local，见 §9.4）。

UPDATE player_scores
SET points_local = NULL, updated_at = now()
WHERE points_local IS NOT NULL;

-- 让已同步过的玩家重新走一遍（幂等，重复执行无副作用）
UPDATE sync_jobs
SET status = 'pending', attempts = 0, next_run_at = now(), updated_at = now()
WHERE kind = 'player';
