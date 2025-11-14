// src/clients/service_client.rs
// (原 common_client.rs)
// 提供了通用的、Nacos 感知的 HTTP 客户端逻辑。

use crate::errors::{AppError};
use crate::state::AppState;
use serde::de::DeserializeOwned; 
use serde::Serialize; 
use tracing::{error, info, warn};

// --- "重载" (Overloads) - 暴露给其他 client 模块的简便函数 ---

/// (最简 GET) 通用 GET 请求，不带查询参数，使用 Nacos 默认分组。
#[allow(dead_code)] // 允许未使用
pub(super) async fn get_service<T>(
    state: &AppState,
    service_name: &str,
    endpoint_path: &str,
) -> Result<T, AppError>
where
    T: DeserializeOwned + 'static,
{
    // 自动调用“完整版”函数，传入 None (group) 和 () (空查询)
    get_service_with_group(state, service_name, None, endpoint_path, &()).await
}

/// (带 Query 的 GET) 通用 GET 请求，带查询参数，使用 Nacos 默认分组。
#[allow(dead_code)] // 允许未使用
pub(super) async fn get_service_with_query<T, Q>(
    state: &AppState,
    service_name: &str,
    endpoint_path: &str,
    query_params: &Q,
) -> Result<T, AppError>
where
    T: DeserializeOwned + 'static,
    Q: Serialize + ?Sized,
{
    // 自动调用“完整版”函数，并传入 None 作为 group_name
    get_service_with_group(state, service_name, None, endpoint_path, query_params).await
}

/// (最简 POST) 通用 POST 请求，只带 Body，使用 Nacos 默认分组。
#[allow(dead_code)] // 允许未使用
pub(super) async fn post_service<T, B>(
    state: &AppState,
    service_name: &str,
    endpoint_path: &str,
    body: &B,
) -> Result<T, AppError>
where
    T: DeserializeOwned + 'static,
    B: Serialize + ?Sized,
{
    // 自动调用“完整版”函数，传入 None (group) 和 () (空查询)
    post_service_with_group(state, service_name, None, endpoint_path, &(), body).await
}

/// (带 Query 的 POST) 通用 POST 请求，带查询参数和 Body，使用 Nacos 默认分组。
#[allow(dead_code)] // 允许未使用
pub(super) async fn post_service_with_query<T, Q, B>(
    state: &AppState,
    service_name: &str,
    endpoint_path: &str,
    query_params: &Q,
    body: &B,
) -> Result<T, AppError>
where
    T: DeserializeOwned + 'static,
    Q: Serialize + ?Sized,
    B: Serialize + ?Sized,
{
    // 自动调用“完整版”函数，并传入 None 作为 group_name
    post_service_with_group(state, service_name, None, endpoint_path, query_params, body).await
}


// --- 核心实现 (现在叫 "with_group") ---

/// (完整版) 通用 GET 请求，可指定 Nacos Group
pub(super) async fn get_service_with_group<T, Q>(
    state: &AppState,
    service_name: &str, // e.g., "rtsp-upms-service"
    group_name: Option<String>, // <-- 允许指定 Group
    endpoint_path: &str,  // e.g., "/check_token"
    query_params: &Q,     // e.g., &[("token", token_str)]
) -> Result<T, AppError>
where
    T: DeserializeOwned + 'static,
    Q: Serialize + ?Sized, 
{
    // 1. 获取基础 URL (现在只通过 Nacos 发现)
    let base_url = discover_service_url(state, service_name, group_name).await?;
    let target_url = format!("{}{}", base_url, endpoint_path);

    info!("(ServiceClient) GET: {}", target_url);

    // 2. 发起 HTTP GET 请求
    let response = state
        .http_client
        .get(target_url)
        .query(query_params) 
        .send()
        .await
        .map_err(|e| {
            error!("(ServiceClient) GET 请求失败: {}", e);
            AppError::InternalError(format!("Failed to call service {}: {}", service_name, e))
        })?;

    // 3. 改进错误处理
    if response.status().is_success() {
        let data = response.json::<T>().await.map_err(|e| {
            error!("(ServiceClient) 解析 GET 响应 JSON 失败: {}", e);
            AppError::InternalError(format!("Failed to parse response from {}: {}", service_name, e))
        })?;
        Ok(data)
    } else {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        warn!(
            "(ServiceClient) 服务 {} GET 返回非 200 状态码: {} | Body: {}",
            service_name, status, error_body
        );
        Err(AppError::InternalError(format!(
            "上游服务 '{}' (路径: '{}') 返回状态码 {}",
            service_name, endpoint_path, status
        )))
    }
}

/// (完整版) 通用 POST 请求，可指定 Nacos Group
pub(super) async fn post_service_with_group<T, Q, B>(
    state: &AppState,
    service_name: &str,
    group_name: Option<String>, // <-- 允许指定 Group
    endpoint_path: &str,
    query_params: &Q, 
    body: &B,
) -> Result<T, AppError>
where
    T: DeserializeOwned + 'static,
    Q: Serialize + ?Sized, 
    B: Serialize + ?Sized,
{
    // 1. 获取基础 URL (逻辑复用)
    let base_url = discover_service_url(state, service_name, group_name).await?;
    let target_url = format!("{}{}", base_url, endpoint_path);

    info!("(ServiceClient) POST: {}", target_url);

    // 2. 发起 HTTP POST 请求
    let response = state
        .http_client
        .post(target_url)
        .query(query_params) 
        .json(body)          
        .send()
        .await
        .map_err(|e| {
            error!("(ServiceClient) POST 请求失败: {}", e);
            AppError::InternalError(format!("Failed to call service {}: {}", service_name, e))
        })?;

    // 3. 改进错误处理
    if response.status().is_success() {
        let data = response.json::<T>().await.map_err(|e| {
            error!("(ServiceClient) 解析 POST 响应 JSON 失败: {}", e);
            AppError::InternalError(format!("Failed to parse response from {}: {}", service_name, e))
        })?;
        Ok(data)
    } else {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        warn!(
            "(ServiceClient) 服务 {} POST 返回非 200 状态码: {} | Body: {}",
            service_name, status, error_body
        );
        Err(AppError::InternalError(format!(
            "上游服务 '{}' (路径: '{}') 返回状态码 {}",
            service_name, endpoint_path, status
        )))
    }
}


/// 辅助函数：封装 Nacos 服务发现
async fn discover_service_url(
    state: &AppState,
    service_name: &str,
    group_name: Option<String>, 
) -> Result<String, AppError> {
    
    // 直接执行 Nacos 服务发现
    let instance = state
        .naming_client
        .select_one_healthy_instance(service_name.to_string(), group_name, Vec::new(), true)
        .await?;
    
    Ok(format!("http://{}:{}", instance.ip, instance.port))
}