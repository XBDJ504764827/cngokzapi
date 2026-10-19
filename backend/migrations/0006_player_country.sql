-- 玩家国家与头像补齐（个人资料页展示国旗 / 真实头像）。
-- 来源：GOKZ.TOP `GET /players/{steamid64}`（国家 ISO 3166 alpha-2、Steam 头像 hash）。

ALTER TABLE players ADD COLUMN IF NOT EXISTS country TEXT;

-- 已同步过的玩家重新走一遍玩家档案，补齐 country / avatar_url。
-- 幂等：重复执行无副作用；162 个玩家任务会在后台按限速慢慢跑完。
UPDATE sync_jobs
SET status = 'pending', attempts = 0, next_run_at = now(), updated_at = now()
WHERE kind = 'player';
