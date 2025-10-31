// src/setup/redis.rs
// 包含所有 Redis 相关的客户端和连接池构建逻辑

use crate::config::app_specific::AppSpecificConfig;
// --- 修改点 ---
// 不再直接 use bb8，而是通过 bb8_redis::bb8 访问
use bb8_redis::{RedisConnectionManager};
use tracing::{info};


/// 构建 Redis 连接池
// --- 修改点 ---
// 更新返回类型，使用完整的 bb8_redis::bb8::Pool 路径
pub async fn build_redis_pool(
    nacos_config: &AppSpecificConfig,
) -> anyhow::Result<bb8_redis::bb8::Pool<RedisConnectionManager>> {
    
    // 1. 从 Nacos 配置中获取 URL
    let redis_config = nacos_config
        .redis
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Nacos 配置中缺少 [redis] 部分"))?;
    
    let redis_url = redis_config
        .url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Nacos 配置 [redis] 中缺少 'url' 字段"))?;

    // 2. 创建 Redis 连接管理器
    let manager = RedisConnectionManager::new(redis_url.clone())?;

    // 3. 创建 bb8 连接池
    // --- 修改点 ---
    // 使用完整的 bb8_redis::bb8::Pool 路径
    let pool = bb8_redis::bb8::Pool::builder()
        .max_size(10) // 设置最大连接数
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)
        .await?;
    
    info!("Redis 连接池创建成功, URL: {}", redis_url);
    Ok(pool)
}

