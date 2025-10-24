// src/middleware/logging.rs
// 定义一个简单的日志中间件。

use axum::{
    body::Body, // <-- 新增：导入 Body 类型
    http::{Request},
    middleware::Next,
    response::Response,
};
use tracing::info;

/// 一个简单的日志中间件
pub async fn log_requests(
    // --- 修改点 ---
    // `Request` 是一个泛型结构体，我们需要指定它的 Body 类型。
    // 在 Axum 中，默认的 Body 类型是 `axum::body::Body`。
    req: Request<Body>,
    next: Next,
) -> Response {
    // 从请求中获取方法和 URI
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    // 1. 在请求到达处理器之前执行的逻辑
    info!("接收到请求 -> 方法: {}, URI: {}", method, uri);

    // 2. 调用 `next.run(req).await` 来执行下一个中间件或处理器
    //    并获取响应
    let response = next.run(req).await;

    // 3. 在响应返回给客户端之前执行的逻辑
    info!("已发送响应 (状态码: {})", response.status());

    // 4. 返回响应
    response
}

