// src/config/mod.rs
// 这是 config 模块的入口文件。
// 负责定义基础配置（通常来自环境变量）和声明子模块。

use std::env;
use thiserror::Error;

// --- 声明子模块 ---
// 告诉编译器，去同级目录下的 "app_specific.rs" 文件加载 app_specific 子模块
pub mod app_specific;

// --- 基础配置 (从环境变量加载) ---

/// 应用的基础配置，从环境变量加载
#[derive(Debug, Clone)]
pub struct Config {
    pub server_addr: String,
    #[allow(dead_code)] // 允许这个字段是“死代码”（未被读取）
    pub database_url: Option<String>,
    pub nacos_addr: String,
    pub nacos_namespace: String,
    pub nacos_username: Option<String>,
    pub nacos_password: Option<String>,
    pub nacos_config_data_id: String,
    pub nacos_config_group: String,
    // --- 新增：Auth 服务的 Nacos 名 ---
    pub auth_service_name: String,
}

/// 配置加载错误枚举 (保持公共)
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Dotenv error: {0}")]
    Dotenvy(#[from] dotenvy::Error),
    #[error("Environment variable error: {0}")]
    Var(#[from] std::env::VarError),
}

impl Config {
    /// 从 .env 文件和环境变量加载基础配置 (保持公共)
    pub fn from_env() -> Result<Self, ConfigError> {
        // 尝试加载 .env 文件，忽略错误（比如文件不存在）
        dotenvy::dotenv().ok();

        let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:4000".to_string());
        let database_url = env::var("DATABASE_URL").ok(); // 可选
        let nacos_addr = env::var("NACOS_ADDR")?;
        let nacos_namespace = env::var("NACOS_NAMESPACE").unwrap_or_default(); // 默认为空字符串 (public)
        let nacos_username = env::var("NACOS_USERNAME").ok(); // 可选
        let nacos_password = env::var("NACOS_PASSWORD").ok(); // 可选
        let nacos_config_data_id = env::var("NACOS_CONFIG_DATA_ID")?;
        let nacos_config_group = env::var("NACOS_CONFIG_GROUP").unwrap_or_else(|_| "DEFAULT_GROUP".to_string());
        // --- 新增：加载 Auth 服务名 ---
        let auth_service_name = env::var("AUTH_SERVICE_NAME")?;
        Ok(Config {
            server_addr,
            database_url,
            nacos_addr,
            nacos_namespace,
            nacos_username,
            nacos_password,
            nacos_config_data_id,
            nacos_config_group,
            auth_service_name
        })
    }
}
