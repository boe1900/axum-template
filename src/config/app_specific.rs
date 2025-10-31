// src/config/app_specific.rs
// 存放从 Nacos 加载的具体业务配置结构体。

use serde::Deserialize; // 需要导入 Deserialize
use serde_yaml; // 需要导入 serde_yaml 来使用其 Error 类型

// --- Nacos 业务配置 (使用嵌套结构体) ---

/// 顶层结构体，对应 Nacos 中的整个 YAML 文件内容
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)] // 暂时允许未使用
pub struct AppSpecificConfig {
    // 对应 YAML 中的 greeting
    pub greeting: Option<String>,
    // 对应 YAML 中的 log_level
    pub log_level: Option<String>,

    // database 字段现在对应 DatabaseConfig 结构体
    pub database: Option<DatabaseConfig>, // <--- 保持或取消注释
    
    // 对应 YAML 中的 feature_flags 嵌套结构
    pub feature_flags: Option<FeatureFlags>,

    // 对应 YAML 中的 service 嵌套结构
    pub service: Option<ServiceConfig>,
}


// --- 新增：数据库配置结构体 ---
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)] // 暂时允许未使用
pub struct DatabaseConfig {
    // 对应 YAML 中的 database.url
    pub url: Option<String>,
    // 对应 YAML 中的 database.pool_size
    pub pool_size: Option<u32>,
}


/// 功能开关配置 (保持，匹配 YAML)
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct FeatureFlags {
    // 对应 YAML 中的 new_dashboard_enabled
    pub new_dashboard_enabled: Option<bool>,
    // 对应 YAML 中的 experimental_feature_x
    pub experimental_feature_x: Option<bool>,
}

/// 服务配置 (保持，匹配 YAML)
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct ServiceConfig {
    // 对应 YAML 中的 timeout_ms
    pub timeout_ms: Option<u64>,
    // 对应 YAML 中的 retry_attempts
    pub retry_attempts: Option<u32>,
}


// --- 解析函数 (保持不变) ---
/// 尝试将从 Nacos 获取的字符串解析为 AppSpecificConfig
pub fn parse_nacos_config(content: &str) -> Result<AppSpecificConfig, serde_yaml::Error> {
    serde_yaml::from_str(content)
}

