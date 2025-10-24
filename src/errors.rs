// src/errors.rs
// 定义统一的错误类型和错误响应。
// 类似于 Spring 的 @ControllerAdvice 和 @ExceptionHandler。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// 定义我们自己的应用错误类型
#[allow(dead_code)]
pub enum AppError {
    // 包装了 SQLx 数据库错误
    DatabaseError(sqlx::Error),
    // 表示资源未找到
    NotFound(String),
    // 其他内部错误
    InternalServerError(String),
}

// 实现 IntoResponse trait，这样我们的 Handler 就可以直接返回 AppError
// Axum 会自动调用这个方法将 AppError 转换为 HTTP 响应
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(e) => {
                // 在真实项目中，这里应该记录详细的错误日志
                tracing::error!("Database error: {}", e);
                // 不应将详细的数据库错误暴露给客户端
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A database error occurred".to_string(),
                )
            }
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::InternalServerError(message) => {
                tracing::error!("Internal server error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, message)
            }
        };

        let body = Json(json!({ "error": error_message }));

        (status, body).into_response()
    }
}

// 这是一个方便的转换，允许我们使用 `?` 运算符
// 将 sqlx::Error 自动转换为我们的 AppError::DatabaseError
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DatabaseError(e)
    }
}
