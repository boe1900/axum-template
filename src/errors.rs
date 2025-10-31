// src/errors.rs
// 定义统一的应用程序错误类型。
// 类似于 Spring Boot 中的 @ControllerAdvice 和 @ExceptionHandler

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
// --- 修改点 ---
// 告诉编译器，`sqlx` crate 现在可以从 `sea_orm` 库中找到
use sea_orm::DbErr;
use serde_json::json;
use thiserror::Error;
use tracing::error;

// --- 新增：导入 Redis 相关的错误类型 ---
use bb8_redis::bb8::RunError;
use redis::RedisError;

/// 统一的应用错误枚举
/// 使用 thiserror 宏可以方便地将其他错误类型转换为 AppError
#[derive(Error, Debug)]
#[allow(dead_code)] // 暂时允许未使用
pub enum AppError {
    #[error("Nacos SDK 错误: {0}")]
    Nacos(#[from] nacos_sdk::api::error::Error),

    #[error("环境变量加载失败: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] DbErr), // <-- 现在 `sqlx::Error` 可以被正确找到了

    #[error("Redis 连接池错误: {0}")]
    RedisPoolError(#[from] RunError<RedisError>),

    // --- 新增：处理 Redis 命令错误 ---
    #[error("Redis 命令错误: {0}")]
    RedisError(#[from] RedisError),

    #[error("Anyhow 错误: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("未找到资源: {0}")]
    NotFound(String),

    #[error("请求无效: {0}")]
    BadRequest(String),

    #[error("未经授权: {0}")]
    Unauthorized(String),
}

/// 实现 IntoResponse trait，让 Axum 知道如何将 AppError 转换为 HTTP 响应
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Nacos(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Nacos error: {}", e)),
            AppError::Config(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Config error: {}", e)),
            AppError::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)),
            AppError::RedisPoolError(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis pool error: {}", e)),
            AppError::RedisError(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis command error: {}", e)),
            AppError::Anyhow(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", e)),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        };

        // 在服务器日志中记录详细错误
        error!("Error: {:?}", error_message);

        // 构建返回给客户端的 JSON 响应
        let body = Json(json!({
            "code": status.as_u16(),
            "message": error_message,
        }));

        (status, body).into_response()
    }
}


