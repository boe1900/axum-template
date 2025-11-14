// src/utils/validated_json.rs
// 定义一个自定义的 Axum 提取器 (Extractor)，
// 它会自动反序列化 JSON 并 *立即* 运行 `validator`。
// 这就是 Rust 版本的 `@Valid` 注解。

// --- 修改点：移除了 async_trait ---
// use async_trait::async_trait; // 👈 删掉
use axum::{
    // 移除了 async_trait
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate; // 导入 Validate trait

use crate::errors::{AppError, ServiceError}; // 导入我们的错误类型

/// 一个自定义提取器，它封装了 `axum::Json`
/// 并在反序列化后自动调用 `.validate()`
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T); // 元组结构体包装

// --- 修改点：移除了 #[async_trait] ---
// Axum 0.7+ 的 `FromRequest` trait 本身就是 `async` 的
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate, // 👈 关键：T 必须能被反序列化和验证
    S: Send + Sync,
    // 确保 Json<T> 也是一个有效的提取器
    Json<T>: FromRequest<S, Rejection = axum::extract::rejection::JsonRejection>,
{
    type Rejection = AppError; // 👈 验证失败时，返回我们统一的 AppError

    // 👈 这个 `async fn` 签名现在 *直接* 匹配 trait (接口) 的定义
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // --- 修正了提取器逻辑 ---
        // 1. 首先，使用 Axum 内置的 Json 提取器来反序列化
        //    我们不调用 req.extract()，而是直接调用 Json<T> 自己的 from_request
        let Json(payload) = Json::<T>::from_request(req, state).await.map_err(|e| {
            // 将 Axum 的 JSON 格式错误转换为我们的业务错误 (10001)
            AppError::Service(ServiceError::InvalidArgument(format!("JSON 格式错误: {}", e)))
        })?;

        // 2. --- 核心步骤 ---
        //    调用 `validator` 库的 .validate() 方法
        payload.validate().map_err(|e| {
            // 3. 如果验证失败 (e.g., 字段为空, 格式错误)
            //    将 `validator` 的 ValidationErrors 转换为我们的业务错误 (10001)
            AppError::Service(ServiceError::InvalidArgument(
                // `e` 包含了所有字段的详细错误信息
                format!("请求参数不合法: {}", e) 
            ))
        })?;

        // 4. 验证成功！
        Ok(ValidatedJson(payload))
    }
}