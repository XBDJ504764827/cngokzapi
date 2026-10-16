//! 编排层：搜索过滤 + 分页。后续在此加 Redis（搜索结果短 TTL）与 PG 回源。

use super::{dto::BansQuery, models::BanEntry, repository};
use crate::{
    errors::AppError,
    modules::page::{self, Page},
};

/// 封禁名单分页查询。搜索不区分大小写，SteamID64 子串匹配。
pub fn page(q: BansQuery) -> Result<Page<BanEntry>, AppError> {
    // TODO: PG 侧 ILIKE + COUNT，这里是内存实现。
    let query = q.query.unwrap_or_default().trim().to_lowercase();
    let all = repository::mock_bans();
    let rows: Vec<BanEntry> = if query.is_empty() {
        all
    } else {
        all.into_iter()
            .filter(|b| b.player.to_lowercase().contains(&query) || b.steam_id64.contains(&query))
            .collect()
    };
    page::paginate(&rows, q.page, q.per_page)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(query: &str, page: u32) -> BansQuery {
        BansQuery {
            query: Some(query.to_string()),
            page: Some(page),
            per_page: None,
        }
    }

    #[test]
    fn paging_and_search() {
        let p1 = page(q("", 1)).unwrap();
        assert_eq!(p1.data.len(), 20);
        assert_eq!(p1.pagination.total, super::repository::BANS_COUNT);
        let p3 = page(q("", 3)).unwrap();
        assert_eq!(p3.data.len(), 5);
        let hit = page(q("progamer", 1)).unwrap();
        assert!(!hit.data.is_empty());
        assert!(page(q("不存在zzz", 1)).unwrap().data.is_empty());
    }
}
