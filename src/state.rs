// src/state.rs
// 定义应用程序的共享状态。
// 类似于 Spring Boot 中 IoC 容器管理的单例 Bean。

// 更新 use 语句以指向新的模块路径
use crate::config::app_specific::AppSpecificConfig;
use nacos_sdk::api::{config::ConfigService, naming::NamingService};
use std::sync::Arc;
use tokio::sync::RwLock;
// --- 修改点 ---
// 导入 SeaORM 的数据库连接类型
use sea_orm::DatabaseConnection;

/// AppState 结构体包含了所有需要在 handlers 之间共享的状态。
#[derive(Clone)]
#[allow(dead_code)] // 暂时允许未使用
pub struct AppState {
    pub naming_client: Arc<NamingService>,
    pub config_client: Arc<ConfigService>,
    // --- 新增字段 ---
    // 添加 app_config 字段来持有从 Nacos 解析的配置
    pub app_config: Arc<RwLock<AppSpecificConfig>>,
    // pub db_pool: PgPool, // 将来添加数据库连接池
    #[allow(dead_code)]
    pub db_pool: Arc<DatabaseConnection>, // <-- 使用 SeaORM 的连接池类型
}

