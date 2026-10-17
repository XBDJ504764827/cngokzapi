//! 通用分页信封：`{ data, pagination: { page, per_page, total } }`（见 PROJECT §6）。

use serde::Serialize;

use crate::errors::AppError;

pub const DEFAULT_PER_PAGE: u32 = 20;
pub const MAX_PER_PAGE: u32 = 100;

#[derive(Serialize)]
pub struct Page<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: Pagination,
}

#[derive(Serialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: usize,
}

/// 校验并归一化分页参数，返回 `(page, per_page, offset)`；各模块按它做 SQL 侧分页。
pub fn normalize(page: Option<u32>, per_page: Option<u32>) -> Result<(u32, u32, i64), AppError> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(DEFAULT_PER_PAGE);
    if page < 1 {
        return Err(AppError::BadRequest("page 从 1 开始".to_string()));
    }
    if !(1..=MAX_PER_PAGE).contains(&per_page) {
        return Err(AppError::BadRequest(format!(
            "per_page 范围 1~{MAX_PER_PAGE}"
        )));
    }
    let offset = i64::from(page - 1) * i64::from(per_page);
    Ok((page, per_page, offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_paging() {
        assert_eq!(normalize(None, None).unwrap(), (1, 20, 0));
        assert_eq!(normalize(Some(3), Some(20)).unwrap(), (3, 20, 40));
        assert!(normalize(Some(0), None).is_err());
        assert!(normalize(Some(1), Some(101)).is_err());
        assert!(normalize(Some(1), Some(0)).is_err());
    }
}
