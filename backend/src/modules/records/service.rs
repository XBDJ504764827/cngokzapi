use super::{models::RecordEntry, repository};

/// 取全部最新记录（60 条）。后续按模式拆小接口或加分页时在此收敛。
pub fn list() -> Vec<RecordEntry> {
    // TODO: 查 PG（ORDER BY created_at DESC LIMIT 60）+ Redis 缓存。
    repository::mock_records()
}
