// src/setup/http.rs
// 包含所有 HTTP 客户端 (reqwest) 相关的构建逻辑

use reqwest::Client;
use std::time::Duration;
use tracing::info;


/// 构建一个共享的 reqwest::Client 实例
///
/// `reqwest::Client` 内部管理着一个连接池，
/// 最佳实践是在整个应用中只创建一次并共享它。
pub fn build_http_client() -> Client {
    info!("正在创建共享 HTTP 客户端 (reqwest::Client)...");
    
    // 你可以在这里为客户端设置全局配置，比如超时
    Client::builder()
        .timeout(Duration::from_secs(10)) // 示例：设置全局 10 秒超时
        .user_agent(format!("axum-template-service/{}", env!("CARGO_PKG_VERSION")))
        .build()
        // Client::new() 在大多数情况下不会失败，所以 unwrap() 是可接受的
        .expect("无法创建共享 HTTP 客户端") 
}