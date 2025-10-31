
// 新增：声明我们的 hello_handler 作为子模块
pub mod hello_handler;

// 将健康检查也移入独立的 handler
pub mod health_handler;


// 新增：KmsAppAccess 处理器
pub mod kms_app_access_handler;

pub mod redis_handler;


// 为了方便 main.rs 调用，我们在这里重导出 health_check 函数
pub use health_handler::health_check;
