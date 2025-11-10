// src/handlers/redis_handler.rs
// 演示如何使用共享的 Redis 连接池

use crate::errors::AppError;
use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use redis::AsyncCommands; // <-- 导入 Redis 异步命令
use serde::Serialize;
use tracing::info;
use crate::response::ApiResponse; // 导入统一响应结构

/// 定义 /redis-test 相关的路由
pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(redis_test_handler))
}

#[derive(Serialize)]
struct RedisTestResponse {
    set_result: String,
    get_result: String,
}

/// GET /
/// 演示从 Redis SET 和 GET
async fn redis_test_handler(
    State(state): State<AppState>, // 注入 AppState
) -> Result<Json<ApiResponse<RedisTestResponse>>, AppError> {
    info!("Handler: redis_test_handler 被调用");

    // 1. 从连接池获取一个连接
    // `.get().await` 可能会失败 (比如超时)
    // 我们在 state.rs 中为 RunError<RedisError> 实现了 From<AppError>
    // 所以我们可以使用 `?` 来自动转换错误
    let mut conn = state.redis_pool.get().await?;
    
    // 2. 执行 SET 命令
    // redis::AsyncCommands 提供了 .set() 和 .get() 方法
    let set_result: String = conn
        .set("my_key", "hello_from_axum_template")
        .await?;

    // 3. 执行 GET 命令
    let get_result: String = conn.get("my_key").await?;
    
    info!("Redis SET result: {}, GET result: {}", set_result, get_result);

     let response_data = RedisTestResponse {
        set_result,
        get_result,
    };
    // --- 修改点：使用 ApiResponse::success 包装 ---
    Ok(Json(ApiResponse::success(response_data)))
}
