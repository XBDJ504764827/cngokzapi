-- 同步任务最后成功时间：enqueue 会覆盖 updated_at，需要独立字段供 /health/sync 使用。
ALTER TABLE sync_jobs ADD COLUMN IF NOT EXISTS last_success_at TIMESTAMPTZ;

-- 回填：当前仍为 done 的任务，用 updated_at 作为最后成功时间。
UPDATE sync_jobs
SET last_success_at = updated_at
WHERE status = 'done' AND last_success_at IS NULL;
