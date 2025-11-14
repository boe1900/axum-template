// src/handlers/kms_app_access_handler.rs
// 负责处理 /app-access/* 相关的 API 请求

use crate::errors::AppError;
use crate::services::kms_app_access_service;
use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::{Deserialize};
use tracing::info;

use crate::middleware::auth::CurrentUser;
use crate::response::ApiResponse; // 导入统一响应结构
use axum::Extension;
use std::sync::Arc;

use crate::middleware::auth::check_permission;

// --- 核心修改点 (1)：导入 `ValidatedJson` 和 `Validate` ---
use crate::utils::validated_json::ValidatedJson;
use validator::Validate; // 导入 `Validate` trait 以使用 `#[derive(Validate)]`


/// 定义 /app-access 相关的路由
/// 这个函数返回一个 Router<AppState>，它会被 main.rs 中的主 Router `nest` (嵌套) 进去
pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        // 映射 GET /:id 到 get_app_access_handler
        .route("/{id}", get(get_app_access_handler))
        .route("/", post(create_app_access_handler))
    // 你可以在这里添加 POST, PUT, DELETE 等路由
}

/// GET /:id 的处理器
///
/// # Arguments
/// * `State(state)` - 提取共享的 AppState
/// * `Path(id)` - 从 URL 路径中提取 id
async fn get_app_access_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Extension(user): Extension<Arc<CurrentUser>>,
) -> Result<Json<ApiResponse<crate::models::kms_app_access::Model>>, AppError> {

    check_permission(&user, "kms_kmsAppAccess_view")?; // 👈 检查权限
    // 你现在可以直接使用 `user` 了！
    info!(
        "Handler: 用户 {} (ID: {}) 正在访问 AppAccess ID: {}",
        user.username, user.id, id
    );

    // 调用 service 层的业务逻辑
    let app_access = kms_app_access_service::get_app_access_by_id(&state, id).await?;

    // 3. --- 修改点：使用 ApiResponse::success 包装 ---
    Ok(Json(ApiResponse::success(app_access)))
}

#[derive(Deserialize, Validate)] // <-- 2. 添加 `Validate`
#[allow(dead_code)]
struct CreateAppAccessRequest {
    // 3. 添加验证注解 (属性)
    // 类似于 Java 的 @NotNull 和 @Size(min=1)
    #[validate(length(min = 1, message = "应用名称(name)不能为空"))]
    name: String,

    // 4. (示例) 添加其他验证
    // 假设 description 最大长度为 2048
    #[validate(length(max = 50, message = "描述(description)长度不能超过 50"))]
    description: Option<String>,
}

/// POST / 的处理器
async fn create_app_access_handler(
    State(_state): State<AppState>, 
    Extension(user): Extension<Arc<CurrentUser>>,
    // --- 核心修改点 (3)：使用 `ValidatedJson` 替代 `Json` ---
    ValidatedJson(payload): ValidatedJson<CreateAppAccessRequest>, // 👈 使用我们自定义的提取器
) -> Result<Json<ApiResponse<()>>, AppError> { // <-- 修改返回类型为 ()
    
    // --- 核心修改点 (4)：权限检查移到这里 ---
    check_permission(&user, "kms_kmsAppAccess_add")?;

    // ---
    // 如果代码能执行到这里，说明：
    // 1. 用户已认证 (mw_require_auth)
    // 2. 权限已检查 (check_permission)
    // 3. JSON body 已被成功反序列化
    // 4. JSON body 已通过了 *所有* `#[validate]` 规则
    // ---
    
    info!("Handler: 用户 {} 正在创建 AppAccess... 名称: {}", user.username, payload.name);
    
    // --- TODO ---
    // 1. 调用 services::kms_app_access_service::create_app_access(&state, payload)
    // 2. service 会将 DTO 转换为 ActiveModel 并调用 repository
    // --- 
    
    // 暂时返回一个模拟的错误，表示“未实现”
    // Err(AppError::InternalError("创建功能尚未实现".to_string()))
    
    // 演示成功（返回一个空的 data 和 200 OK）
    Ok(Json(ApiResponse::success(())))
}