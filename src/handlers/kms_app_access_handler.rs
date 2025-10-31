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
) -> Result<Json<crate::models::kms_app_access::Model>, AppError> {
    info!("Handler: get_app_access_handler 被调用, ID: {}", id);

    // 调用 service 层的业务逻辑
    let app_access = kms_app_access_service::get_app_access_by_id(&state, id).await?;

    // 如果 service 返回 Ok(app_access)，
    // Axum 会自动将其序列化为 JSON 并返回 200 OK
    // 如果 service 返回 Err(AppError)，
    // `?` 会提前返回，并触发 AppError 的 IntoResponse 实现
    Ok(Json(app_access))
}
