use crate::state::AppState; // 导入共享状态结构体
use axum::{
    extract::State, // 用于提取共享状态
    routing::get,   // 用于定义 GET 路由
    Json,           // <-- 新增：用于返回 JSON 响应
    Router,         // Axum 的路由器类型
};
use serde::Serialize; // <-- 新增：用于派生 Serialize
use tracing::info; // 导入日志宏
use crate::response::ApiResponse; // 导入统一响应结构
use crate::errors::AppError; // 导入错误类型


/// 定义与 "hello" 相关的路由
pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        // 将路径 "/" 映射到 hello_from_nacos_config 处理器
        .route("/", get(hello_from_nacos_config))
}

// --- 新增：定义 JSON 响应的结构体 ---
#[derive(Serialize)] // 派生 Serialize 以便转换为 JSON
struct HelloResponse {
    message: String,
    log_level: Option<String>,
    dashboard_enabled: Option<bool>,
}

/// 一个从 AppState 读取 Nacos 配置并返回 JSON 的 handler
async fn hello_from_nacos_config(
    State(state): State<AppState>, // 使用 State 提取器来获取共享状态
) -> Result<Json<ApiResponse<HelloResponse>>, AppError> { // <-- 修改：返回类型为 Json<HelloResponse>
    info!("Handler: hello_from_nacos_config 被调用");

    // --- 从 AppState 读取已解析的配置 ---
    // 使用 .read().await 获取读锁 (因为 RwLock 是异步的)
    let config_guard = state.app_config.read().await;

    // 从锁守卫中安全地访问配置字段
    // 我们使用 .clone() 来复制 Option<String> 内部的 String (如果存在)
    // 或者直接克隆 Option 本身 (对于 bool 来说开销很小)
    let greeting_message = config_guard
        .greeting
        .clone()
        .unwrap_or_else(|| "Default Greeting from Code".to_string()); // 提供默认值

    let log_level_config = config_guard.log_level.clone(); // 克隆 Option<String>

    // 访问嵌套的 feature_flags
    let dashboard_enabled_config = config_guard
        .feature_flags
        .as_ref() // 获取 Option<&FeatureFlags>
        .and_then(|flags| flags.new_dashboard_enabled); // 获取 Option<bool>

    info!(
        "Handler: 读取到配置 - greeting: {}, log_level: {:?}, dashboard: {:?}",
        greeting_message, log_level_config, dashboard_enabled_config
    );

    // 构建响应结构体
    let response = HelloResponse {
        message: greeting_message,
        log_level: log_level_config,
        dashboard_enabled: dashboard_enabled_config,
    };

    // --- 修改点：使用 ApiResponse::success 包装 ---
    Ok(Json(ApiResponse::success(response)))
}