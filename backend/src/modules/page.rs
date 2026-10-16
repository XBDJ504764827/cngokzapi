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

/// 归一化分页参数并切片。`page` 从 1 起，`per_page` 越界直接 400。
pub fn paginate<T: Clone + Serialize>(
    rows: &[T],
    page: Option<u32>,
    per_page: Option<u32>,
) -> Result<Page<T>, AppError> {
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
    let start = ((page - 1) * per_page) as usize;
    let data = rows
        .iter()
        .skip(start)
        .take(per_page as usize)
        .cloned()
        .collect();
    Ok(Page {
        data,
        pagination: Pagination {
            page,
            per_page,
            total: rows.len(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_and_clamps() {
        let rows: Vec<u32> = (1..=45).collect();
        let p1 = paginate(&rows, Some(1), None).unwrap();
        assert_eq!(p1.data.len(), 20);
        assert_eq!(p1.pagination.total, 45);
        let p3 = paginate(&rows, Some(3), None).unwrap();
        assert_eq!(p3.data.len(), 5);
        assert!(paginate::<u32>(&rows, Some(0), None).is_err());
        assert!(paginate::<u32>(&rows, Some(1), Some(101)).is_err());
    }
}
