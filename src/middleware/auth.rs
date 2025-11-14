// src/middleware/auth.rs
// 负责认证和授权

use crate::errors::{AppError, ServiceError};
use crate::state::AppState;
use axum::{
    // --- 修改点 ---
    // 移除了 FromRequestParts 和 Parts
    extract::{Request, State},
    http::{HeaderMap},
    middleware::Next,
    response::Response,
};
// --- 修改点 ---
// 移除了 async_trait，因为我们不再需要它了
// use async_trait::async_trait; 
use crate::clients::auth_client; 
use std::sync::Arc;
use tracing::{warn};


// --- 2. 定义我们自己的 CurrentUser 结构体 (保持不变) ---
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CurrentUser {
    pub id: String,
    pub username: String,
    pub permissions: Vec<String>,
}

// --- 4. 认证中间件 (核心逻辑) ---
/// 认证中间件 (mw_require_auth)
pub async fn mw_require_auth(
    State(state): State<AppState>, // <-- 注入 AppState
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    
    // 1. 提取 Token
    let token = extract_token(req.headers())?;

    // 2. --- 核心修改点 ---
    // 调用封装好的 Auth 客户端
    let auth_response = auth_client::check_token(&state, &token).await?;
    
    // 3. 将 AuthResponse 转换为 CurrentUser
    //    (auth_response 现在是 auth_client 内部定义的类型)
    let permissions = auth_response.user_info.authorities
        .into_iter()
        .map(|auth| auth.authority)
        .collect();

    let current_user = CurrentUser {
        id: auth_response.user_info.user_id,
        username: auth_response.user_info.username,
        permissions,
    };

    // 4. 存入 extensions (这部分保持不变)
    req.extensions_mut().insert(Arc::new(current_user));

    // 5. 放行
    Ok(next.run(req).await)
}

// --- 5. 辅助函数 (保持不变) ---

/// 辅助函数：从 Headers 中提取 Bearer Token
fn extract_token(headers: &HeaderMap) -> Result<String, AppError> {
    let header_value = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            warn!("认证中间件：Header 'Authorization' 缺失");
            ServiceError::Unauthorized
        })?;

    if !header_value.starts_with("Bearer ") {
        warn!("认证中间件：Header 'Authorization' 格式错误，非 Bearer");
        return Err(ServiceError::Unauthorized.into());
    }
    
    Ok(header_value[7..].to_string())
}
