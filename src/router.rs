// src/router.rs
// 负责组装所有的 Axum 路由和顶层中间件

use crate::state::AppState;
use axum::{
    Router, // ServiceExt 不再需要在这里导入
    middleware as axum_middleware,
    routing::get,
};

/// 创建并组装所有的 Axum 路由
// --- 修改点 ---
// 移除了 async 关键字，这是一个同步函数
pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        // 挂载健康检查
        .route("/", get(crate::handlers::health_check))
        // 挂载 /hello 路由
        .nest("/hello", crate::handlers::hello_handler::routes())
        // 挂载 /app-access 路由
        .nest("/app-access",crate::handlers::kms_app_access_handler::routes(),)
        .nest("/redis-test", crate::handlers::redis_handler::routes())
        // (将来在这里添加更多 .nest() ...)
        // 注入共享状态
        .with_state(app_state)
        // 应用全局日志中间件
        .layer(axum_middleware::from_fn(
            crate::middleware::logging::log_requests,
        ))
}
