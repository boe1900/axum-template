// src/main.rs
// 这是应用程序的主入口，负责“组装”所有部分并启动服务器。
// 类似于 Spring Boot 的主 Application 类。

// 声明我们项目中的其他模块
mod config; // <-- 这里声明顶层 config 模块
mod errors;
mod handlers;
mod middleware;
mod models;
mod repository;
mod services;
mod state;
mod setup;
mod router; // <-- 声明 router 模块
mod response;



// 更新 use 语句以指向新的模块路径
// 只需要导入最核心的 Config 和 setup, router 模块
use crate::config::Config;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,axum_template=debug,nacos_sdk=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. 加载基础配置 (来自 config/mod.rs)
    let config = Config::from_env()?;

    // 3. 初始化应用状态 (Nacos, 数据库, 配置监听)
    info!("正在初始化应用状态 (Nacos, 数据库, 配置...)");
    let app_state = setup::setup_application_state(&config).await?;
    info!("应用状态初始化完成");

    // 4. 注册服务实例到 Nacos
    setup::register_nacos_instance(&config, &app_state.naming_client).await?;

    // 5. --- 修改点 ---
    // 创建 Axum 路由 (这是一个同步操作，移除 .await)
    let app = router::create_router(app_state); // <-- 移除了 .await
    info!("Axum 路由组装完成");

    // 6. 启动服务器 (逻辑仍在 setup 模块)
    info!("服务器即将启动在: {}", &config.server_addr);
    setup::run_server(app, &config.server_addr).await?;

    Ok(())
}
















