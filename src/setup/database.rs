
use crate::config::app_specific::AppSpecificConfig;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tracing::info;



/// 构建数据库连接池，仅使用 Nacos 配置
pub async fn build_db_pool(
    nacos_config: &AppSpecificConfig, // Nacos 配置 (用于获取 URL)
) -> anyhow::Result<DatabaseConnection> {
    
    // 1. 从 Nacos 配置中获取 URL
    let db_config = nacos_config
        .database
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Nacos 配置中缺少 [database] 部分"))?;
    
    let db_url = db_config
        .url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Nacos 配置 [database] 中缺少 'url' 字段"))?;

    // 2. --- 关键步骤：移除 shellexpand ---
    // 直接使用从 Nacos 获取的 db_url
    
    // 3. 使用 URL 创建连接池
    let max_connections = db_config.pool_size.unwrap_or(5);

    let mut opt = ConnectOptions::new(db_url.to_owned()); // <-- 直接使用 db_url
    opt.max_connections(max_connections)
       .min_connections(1)
       .connect_timeout(Duration::from_secs(8))
       .idle_timeout(Duration::from_secs(8))
       .sqlx_logging(true)
       .sqlx_logging_level(tracing::log::LevelFilter::Debug);

    info!("正在连接数据库, 最大连接数: {}", max_connections);
    // (日志中不再打印 db_url，因为它可能包含明文密码)
    let pool = Database::connect(opt).await?;
    
    Ok(pool)
}