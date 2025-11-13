// src/handlers/kms_app_access_handler.rs
// 负责处理 /app-access/* 相关的 API 请求

use crate::errors::AppError;
use crate::services::kms_app_access_service;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use tracing::info;

use crate::response::ApiResponse; // 导入统一响应结构
use axum::Extension;
use crate::middleware::auth::CurrentUser;
use std::sync::Arc;



/// 定义 /app-access 相关的路由
/// 这个函数返回一个 Router<AppState>，它会被 main.rs 中的主 Router `nest` (嵌套) 进去
pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        // 映射 GET /:id 到 get_app_access_handler
        .route("/{id}", get(get_app_access_handler))
    // 你可以在这里添加 POST, PUT, DELETE 等路由
    // .route("/", post(create_app_access_handler))
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
