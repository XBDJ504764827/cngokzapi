//! 编排层：搜索 + 分页。数据由 `modules/sync` 从上游导入本站 PG。

use super::{dto::BansQuery, models::BanEntry, repository};
use crate::{
    errors::AppError,
    modules::page::{self, Page, Pagination},
    state::AppState,
};

/// 封禁名单分页查询。搜索不区分大小写，SteamID64 子串匹配。
pub async fn page(state: &AppState, q: BansQuery) -> Result<Page<BanEntry>, AppError> {
    let (page_no, per_page, offset) = page::normalize(q.page, q.per_page)?;
    let query = q.query.unwrap_or_default().trim().to_string();
    let (data, total) = repository::search(&state.db, &query, i64::from(per_page), offset).await?;
    Ok(Page {
        data,
        pagination: Pagination {
            page: page_no,
            per_page,
            total,
        },
    })
}
