//! 统一错误处理：所有接口错误都收敛为 `{ error: { code, message } }`（见 PROJECT §6）。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// 业务错误。新增变体时同步补充下面的 `(状态码, code)` 映射与单测。
/// auth / admin 模块落地前部分变体暂无人构造，allow 随之消除。
#[allow(dead_code)]
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound(String),
    /// 依赖未就绪（如无 PG/Redis 时的占位数据访问失败）。
    ServiceUnavailable(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(m) => write!(f, "参数错误：{m}"),
            AppError::Unauthorized => write!(f, "需要先登录"),
            AppError::Forbidden => write!(f, "无权限"),
            AppError::NotFound(m) => write!(f, "不存在：{m}"),
            AppError::ServiceUnavailable(m) => write!(f, "依赖不可用：{m}"),
            AppError::Internal(m) => write!(f, "服务内部错误：{m}"),
        }
    }
}

impl std::error::Error for AppError {}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "INVALID_PARAM", m),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "需要先登录".to_string(),
            ),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", "无权限".to_string()),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m),
            AppError::ServiceUnavailable(m) => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", m)
            }
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", m),
        };
        (
            status,
            Json(ErrorBody {
                error: ErrorDetail { code, message },
            }),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = %e, "database error");
        AppError::Internal("数据库异常".to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        tracing::error!(error = %e, "redis error");
        AppError::ServiceUnavailable("缓存服务暂不可用".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_mapping() {
        // 全变体映射回归：参数错 400、未登录 401、无权限 403、不存在 404、依赖缺失 503。
        for (err, expect) in [
            (
                AppError::BadRequest("x".to_string()),
                StatusCode::BAD_REQUEST,
            ),
            (AppError::Unauthorized, StatusCode::UNAUTHORIZED),
            (AppError::Forbidden, StatusCode::FORBIDDEN),
            (AppError::NotFound("x".to_string()), StatusCode::NOT_FOUND),
            (
                AppError::ServiceUnavailable("x".to_string()),
                StatusCode::SERVICE_UNAVAILABLE,
            ),
        ] {
            assert_eq!(err.into_response().status(), expect);
        }
    }
}
