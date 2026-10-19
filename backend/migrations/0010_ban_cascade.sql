-- 封禁级联补数据：对新同步逻辑上线前已导入的封禁，统一失效其成绩并移出榜单。
-- 之后由 `run_bans` 增量任务实时处理新封禁/解禁。

UPDATE records
SET is_valid = FALSE, updated_at = now()
WHERE is_valid
  AND player_steam_id IN (SELECT steam_id FROM bans);

DELETE FROM player_scores
WHERE player_steam_id IN (SELECT steam_id FROM bans);
