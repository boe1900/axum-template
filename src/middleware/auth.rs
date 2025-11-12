// src/middleware/auth.rs
// 负责认证和授权

use crate::errors::{AppError, ServiceError};
use crate::state::AppState;
use axum::{
    extract::{Request, State}, 
    http::{HeaderMap},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{info, warn, error};
use serde::Deserialize;

// --- 1. 定义 Rust 结构体来精确匹配 Auth 服务的 JSON 响应 ---
#[derive(Debug, Deserialize)]
struct AuthResponse {
    user_info: UserInfo,
}
#[derive(Debug, Deserialize, Clone)]
struct UserInfo {
    #[serde(rename = "id")]
    user_id: String,
    username: String,
    authorities: Vec<Authority>,
}
#[derive(Debug, Deserialize, Clone)]
struct Authority {
    authority: String,
}


// --- 2. 定义我们自己的 CurrentUser 结构体 (保持不变) ---
#[derive(Debug, Clone)]
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

    // 2. 调用 Auth 服务
    let auth_response = check_token_with_auth_service(&state, &token).await?;
    
    // 3. 将 AuthResponse 转换为 CurrentUser
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

/// 辅助函数：调用 Auth 服务
async fn check_token_with_auth_service(
    state: &AppState,
    token: &str,
) -> Result<AuthResponse, AppError> {
    
    // 1. 从 Nacos 发现 Auth 服务的实例
    let auth_service_name = &state.base_config.auth_service_name;
    let group_name = None; // 依赖默认 Group

    // --- 修改点 ---
    // .await? 会在 Nacos 找不到服务时返回 Err，并被 `?` 自动转换为 AppError
    // `instance` 在这里 *一定* 是 `ServiceInstance`
    let instance = state
        .naming_client
        .select_one_healthy_instance(
            auth_service_name.to_string(), 
            group_name,
            Vec::new(),   // clusters
            true          // healthy
        )
        .await?;

    let auth_service_url = format!("http://{}:{}/token/check_token", instance.ip, instance.port);
    info!("正在调用 Auth 服务: {}", auth_service_url);

    // 2. 发起 HTTP GET 请求
    let response = state
        .http_client
        .get(auth_service_url)
        .query(&[("token", token)]) // 作为 URL 查询参数
        .send()
        .await
        .map_err(|e| {
            error!("调用 Auth 服务失败: {}", e);
            AppError::InternalError(format!("Failed to call auth service: {}", e))
        })?;

    // 3. 处理响应
    if response.status().is_success() {
        // HTTP 200 OK -> 尝试解析 JSON
        let auth_response = response.json::<AuthResponse>().await.map_err(|e| {
            error!("解析 Auth 服务 JSON 响应失败: {}", e);
            AppError::InternalError(format!("Failed to parse auth response: {}", e))
        })?;
        Ok(auth_response)
    } else {
        // HTTP 401, 403, 500 等 -> 认证失败
        warn!("Auth 服务返回非 200 状态码: {}", response.status());
        Err(ServiceError::Unauthorized.into())
    }
}