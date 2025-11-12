// src/state.rs
// 定义应用程序的共享状态。
// 类似于 Spring Boot 中 IoC 容器管理的单例 Bean。

// 更新 use 语句以指向新的模块路径
use crate::config::app_specific::AppSpecificConfig;
use crate::config::Config; // <-- 新增：导入基础配置
use nacos_sdk::api::{config::ConfigService, naming::NamingService};
use std::sync::Arc;
use tokio::sync::RwLock;
// --- 修改点 ---
// 导入 SeaORM 的数据库连接类型
use sea_orm::DatabaseConnection;

// 从 `bb8_redis` 中导入 `bb8` 提供的类型
use bb8_redis::RedisConnectionManager;
use bb8_redis::bb8::Pool;
use reqwest::Client; // <-- 新增：导入 reqwest 客户端


/// AppState 结构体包含了所有需要在 handlers 之间共享的状态。
#[derive(Clone)]
#[allow(dead_code)] // 暂时允许未使用
pub struct AppState {
    // --- 新增：持有基础配置 ---
    pub base_config: Arc<Config>,

    pub naming_client: Arc<NamingService>,
    pub config_client: Arc<ConfigService>,
    // --- 新增字段 ---
    // 添加 app_config 字段来持有从 Nacos 解析的配置
    pub app_config: Arc<RwLock<AppSpecificConfig>>,
    // pub db_pool: PgPool, // 将来添加数据库连接池
    pub db_pool: DatabaseConnection, // <-- 使用 SeaORM 的连接池类型

    pub redis_pool: Pool<RedisConnectionManager>,

    pub http_client: Client,
}
