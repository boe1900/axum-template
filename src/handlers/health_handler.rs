// src/handlers/health_handler.rs
// 提供一个简单的健康检查端点。

use crate::response::ApiResponse; // <-- 导入 ApiResponse
use axum::Json;
use serde::Serialize;

// --- 新增：健康检查的响应体 ---
#[derive(Serialize)]
pub struct HealthCheckResponse {
    status: String,
}

/// 一个简单的健康检查处理器
pub async fn health_check() -> Json<ApiResponse<HealthCheckResponse>> {
    let response = HealthCheckResponse {
        status: "UP".to_string(),
    };
    // 使用 ApiResponse::success 包装
    Json(ApiResponse::success(response))
}
