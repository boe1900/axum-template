// src/handlers/health_handler.rs
// 提供一个简单的健康检查端点。

use axum::http::StatusCode;
use axum::response::IntoResponse; // 用于将元组转换为 Response

/// 一个简单的健康检查处理器
// 返回一个实现了 IntoResponse 的类型，这里是一个元组 (StatusCode, String)
pub async fn health_check() -> impl IntoResponse {
    // 返回 HTTP 状态码 200 OK 和响应体 "OK"
    (StatusCode::OK, "OK".to_string())
}

