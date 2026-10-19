//! 编排层：参数校验 + 分页榜单查询。榜单数据由 `modules/sync` 定时/按需同步进 PG。

use super::{
    dto::LeaderboardQuery,
    models::{LeaderboardPage, Pagination},
    repository,
};
use crate::{errors::AppError, modules::mode, state::AppState};

/// 分页榜单：搜索 / 活跃度 / 排序全部下推到 SQL。
pub async fn board(state: &AppState, q: &LeaderboardQuery) -> Result<LeaderboardPage, AppError> {
    let m = mode::validate(q.mode.as_deref())?;
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q
        .per_page
        .unwrap_or(repository::DEFAULT_PAGE_SIZE)
        .clamp(1, repository::MAX_PAGE_SIZE);
    let sort = match q.sort.as_deref() {
        Some("maps") => "maps",
        Some("avg") => "avg",
        Some("rating") => "rating",
        _ => "points",
    };
    let activity = match q.activity.as_deref() {
        Some("7") => "7",
        Some("30") => "30",
        Some("inactive") => "inactive",
        _ => "all",
    };
    let query = q.query.as_deref().unwrap_or("").trim().to_string();
    let offset = (page - 1) * per_page;

    let (data, total) =
        repository::page(&state.db, m, &query, activity, sort, per_page, offset).await?;

    Ok(LeaderboardPage {
        data,
        pagination: Pagination {
            page,
            per_page,
            total,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_mode() {
        assert!(mode::validate(Some("XXX")).is_err());
        assert_eq!(mode::validate(None).unwrap(), "KZT");
        assert_eq!(mode::validate(Some("VNL")).unwrap(), "VNL");
    }
}
