// src/clients/mod.rs
// 声明所有微服务客户端

// --- 修改点 ---
// 重命名为 service_client
mod service_client;

// -------------------------------------
// --- 具体的业务客户端 ---
// -------------------------------------

// auth_client 是公开的，可以被 middleware 使用
pub mod auth_client;

// (将来你可以添加 pub mod user_client; 等等)