// src/router.rs
// 负责组装所有的 Axum 路由和顶层中间件

use crate::state::AppState;
use axum::{
    middleware::{self as axum_middleware}, // <-- 重命名
    routing::get,
    Router,
};
// 导入我们自定义的认证中间件
use crate::middleware::auth::mw_require_auth;


/// 创建并组装所有的 Axum 路由
pub fn create_router(app_state: AppState) -> Router {
    // --- 1. 构建需要“认证”的路由 ---
    // 这些路由会先经过 mw_require_auth 中间件
    let protected_routes = Router::new()
        .nest("/hello", crate::handlers::hello_handler::routes())
        .nest(
            "/app-access",
            crate::handlers::kms_app_access_handler::routes(),
        )
        .nest("/redis-test", crate::handlers::redis_handler::routes())
        // (将来所有需要登录的业务路由都加在这里)
        
        // --- 核心修改点 ---
        // 我们必须使用 `from_fn_with_state` 来包装需要 AppState 的中间件
        .route_layer(axum_middleware::from_fn_with_state(
            app_state.clone(), // 👈 1. 将 AppState 的克隆传递给中间件
            mw_require_auth    // 👈 2. 传递我们的中间件函数
        ));


    // --- 2. 构建“公共”路由 ---
    // 这些路由 *不* 需要认证
    let public_routes = Router::new()
        .route("/", get(crate::handlers::health_check));
        // (将来比如 /login, /metrics, /docs 等路由放这里)


    // --- 3. 组装总路由 ---
    Router::new()
        .merge(public_routes) // 合并公共路由
        .merge(protected_routes) // 合并受保护的路由
        
        // 注入共享状态 (对所有路由生效)
        // (这个 .with_state() 负责将 AppState 注入给 *Handler*，
        //  而 .route_layer(from_fn_with_state...) 负责将其注入给 *Middleware*)
        .with_state(app_state)
        
        // 应用全局日志中间件 (对所有路由生效)
        .layer(axum_middleware::from_fn(
            crate::middleware::logging::log_requests,
        ))
}