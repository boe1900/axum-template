// src/setup/mod.rs
// 这个文件现在是 setup 目录模块的根
// 它的职责是声明子模块，并“重导出” (re-export) 公共函数

// 1. 声明子模块
pub mod database;
pub mod nacos;
pub mod redis;


// 2. 重导出子模块的公共函数
pub use nacos::{
    register_nacos_instance, AppConfigChangeListener,
};

use crate::config::Config;
use crate::state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;
use axum::Router;
use crate::config::app_specific::parse_nacos_config;



// --- 封装所有启动逻辑的主函数 ---
/// 初始化所有应用服务（Nacos 客户端、数据库池、配置加载和监听）
/// 并返回一个构建好的 AppState
pub async fn setup_application_state(config: &Config) -> anyhow::Result<AppState> {
    // 创建 Nacos 客户端
    info!("正在连接 Nacos: {}", &config.nacos_addr);
    let naming_client = Arc::new(nacos::build_nacos_naming_client(config)?);
    let config_client = Arc::new(nacos::build_nacos_config_client(config)?);

    // 获取初始 Nacos 配置并解析
    let initial_config_resp = config_client
        .get_config(config.nacos_config_data_id.clone(), config.nacos_config_group.clone())
        .await?;
    info!("从 Nacos 获取到初始 ConfigResponse: {:?}", initial_config_resp);

    let initial_app_config = parse_nacos_config(&initial_config_resp.content())
        .expect("无法解析初始 Nacos 配置！请检查 Nacos 中的配置格式。");
    info!("成功解析初始 Nacos 配置: {:?}", initial_app_config);

    // 并行构建 DB 和 Redis 连接池
    info!("正在并行创建数据库和 Redis 连接池...");
    let (db_pool_result, redis_pool_result) = tokio::join!(
        database::build_db_pool(&initial_app_config),
        redis::build_redis_pool(&initial_app_config)
    );

    let db_pool = Arc::new(db_pool_result?);
    let redis_pool = Arc::new(redis_pool_result?);
    info!("数据库和 Redis 连接池创建成功");

    // 将解析后的配置放入 RwLock
    let app_config_rwlock = Arc::new(RwLock::new(initial_app_config));


     // 创建 AppState
    let app_state = AppState {
        naming_client: naming_client.clone(),
        config_client: config_client.clone(),
        app_config: app_config_rwlock.clone(),
        db_pool: db_pool.clone(),
        redis_pool: redis_pool.clone(),
    };

    // 添加配置监听器
    config_client
        .add_listener(
            config.nacos_config_data_id.clone(),
            config.nacos_config_group.clone(),
            Arc::new(AppConfigChangeListener { 
                app_config: app_state.app_config.clone()
            }),
        )
        .await?;
    info!("已添加 Nacos 配置监听器");
// 7. 返回构建好的 AppState
    Ok(app_state)
}


// --- 封装 Axum 服务器启动 (保持不变) ---
/// 绑定端口并启动 Axum Web 服务器
pub async fn run_server(app: Router, server_addr: &str) -> anyhow::Result<()> {
    let addr = server_addr.parse::<SocketAddr>()?;
    info!("服务器已启动，正在监听: http://{}", &addr);
    
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}