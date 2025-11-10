// src/errors.rs
// 定义统一的应用程序错误类型。
// 类似于 Spring Boot 中的 @ControllerAdvice 和 @ExceptionHandler

use crate::response::ApiResponse; // <-- 导入我们统一的响应结构
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
// --- 修改点 ---
// 告诉编译器，`sqlx` crate 现在可以从 `sea_orm` 库中找到
use sea_orm::DbErr;
use thiserror::Error;
use tracing::error;

// --- 新增：导入 Redis 相关的错误类型 ---
use bb8_redis::bb8::RunError;
use redis::RedisError;

// --- 新增：业务错误枚举 (等同于你的 CommonCodeEnum) ---
// 它定义了所有“可预期的”业务逻辑错误
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ServiceError {
    // 类似于 10001 INVALID_ARGUMENT
    #[error("请求参数不合法: {0}")]
    InvalidArgument(String), // 允许传入自定义消息
    // 类似于 10004 RESOURCE_NOT_FOUND
    #[error("请求的资源不存在")]
    ResourceNotFound, // 使用默认消息
    // 对应 10003 FORBIDDEN
    #[error("无权访问该资源: {0}")]
    Forbidden(String), // 允许传入需要什么权限
}

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

    // --- 业务错误 (1xxxx) ---
    // AppError::Service(ServiceError) 就等同于 Java 的 BusinessException(ErrorCode)
    #[error("{0}")] // 让 ServiceError 的 #[error] 消息透传出来
    Service(#[from] ServiceError),
}

/// 实现 IntoResponse trait，让 Axum 知道如何将 AppError 转换为 HTTP 响应
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 1. 将 AppError 映射到一个元组 (HTTP 状态码, 自定义业务码, 错误消息)
        //    我们在这里严格遵循你提供的 CommonCodeEnum 规范
        let (status_code, business_code, error_message) = match self {
            // --- 核心修改：处理 ServiceError (业务异常) ---
            AppError::Service(err) => {
                // `err` 就是 ServiceError
                // 我们在这里匹配具体的业务错误码
                let (status, code) = match err {
                    // 对应 10001
                    ServiceError::InvalidArgument(_) => (StatusCode::BAD_REQUEST, 10001),
                    // 对应 10002
                    ServiceError::Forbidden(_) => (StatusCode::FORBIDDEN, 10003),
                    // 对应 10004
                    ServiceError::ResourceNotFound => (StatusCode::NOT_FOUND, 10004),
                };
                // err.to_string() 会自动使用 ServiceError 上定义的 #[error] 消息
                // 比如 "操作重复: 用户名 'admin' 已存在"
                (status, code, err.to_string())
            }

            // ========== 服务端错误 (2xxxx) ==========
            // 对应 20003
            AppError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                20003,
                format!("数据库服务异常: {}", e),
            ),
            // 对应 20002
            AppError::RedisPoolError(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                20002,
                format!("Redis 连接池服务暂不可用: {}", e),
            ),

            // 对应 20001 (通用服务端错误)
            AppError::RedisError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                20001,
                format!("Redis 命令执行错误: {}", e),
            ),
            AppError::Config(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                20001,
                format!("服务器配置加载错误: {}", e),
            ),
            AppError::Anyhow(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                20001,
                format!("服务器内部未知错误: {}", e),
            ),

            // ========== 第三方服务错误 (3xxxx) ==========
            // 对应 30001
            AppError::Nacos(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                30001,
                format!("Nacos SDK 错误: {}", e),
            ),
        };

        // 在服务器日志中记录详细错误
        error!(
            "Error: (HTTP {}) (BizCode {}) - {:?}",
            status_code, business_code, error_message
        );

        // 2. 构建返回给客户端的统一 ApiResponse (data 为 None)
        let body = Json(ApiResponse::<()>::error(business_code, error_message));

        // 3. 返回最终的 HTTP 响应 (使用 HTTP 状态码 和 JSON body)
        (status_code, body).into_response()
    }
}
