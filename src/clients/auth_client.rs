// src/clients/auth_client.rs
// 专门负责与 Auth 服务 (rtsp-upms-service) 通信

use crate::errors::{AppError, ServiceError};
use crate::state::AppState;
use serde::Deserialize;
use tracing::{info, warn};

// --- 修改点 ---
// 导入通用的客户端
use super::service_client;

// --- 1. 定义此客户端需要的数据结构 (DTOs) ---
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub user_info: UserInfo,
}
#[derive(Debug, Deserialize, Clone)]
pub struct UserInfo {
    #[serde(rename = "id")]
    pub user_id: String,
    pub username: String,
    pub authorities: Vec<Authority>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Authority {
    pub authority: String,
}

// --- 2. 定义我们的 "Feign 客户端" 方法 ---

/// 封装调用 Auth 服务的 check_token 接口
pub async fn check_token(
    state: &AppState,
    token: &str,
) -> Result<AuthResponse, AppError> {
    
    info!("(AuthClient) 正在调用 check_token...");

    // 1. 从 AppState 中 *获取* 实际的服务名
    let auth_service_name = &state.base_config.auth_service_name;

    // 2. 定义查询参数
    let query_params = [("token", token)];

    // 3. --- 核心修改点 ---
    // 调用 *简便版* 的 GET (get_service_with_query)，它会自动使用 `None` 作为 group
    let result = service_client::get_service_with_query(
        state,
        auth_service_name, 
        "/token/check_token",
        &query_params,       
    )
    .await;

    // --- 错误处理 (保持不变) ---
    match result {
        Ok(auth_response) => Ok(auth_response), // 成功
        Err(AppError::InternalError(msg)) => {
            warn!("(AuthClient) Auth 服务调用失败 (可能 Token 无效): {}", msg);
            Err(ServiceError::Unauthorized.into()) // 转换为业务上的“未授权”
        }
        Err(e) => {
            Err(e)
        }
    }
}