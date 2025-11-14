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
mod clients;



// 更新 use 语句以指向新的模块路径
// 只需要导入最核心的 Config 和 setup, router 模块
use crate::config::Config;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// --- 修改点：导入 Tokio 信号处理 ---
use tokio::signal;
// --- 新增：导入 Unix 信号处理 (用于 SIGTERM) ---
// `cfg(unix)` 确保这行代码只在 Linux/macOS (包括 WSL, Docker) 上编译
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use cfg_if::cfg_if;


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
    let app = router::create_router(app_state.clone()); // <-- 移除了 .await
    info!("Axum 路由组装完成");

    // 6. --- 修改点：定义更健壮的优雅停机信号 ---
    let shutdown_naming_client = app_state.naming_client.clone();
    let shutdown_config = config.clone(); // Config 必须 derive(Clone)

    //核心修改点：使用 cfg_if! 宏 ---
    let shutdown_signal = async move{
        cfg_if! {
            // 检查是否在 Unix 平台 (Linux, macOS, WSL)
            if #[cfg(unix)] {
                info!("正在监听 Ctrl+C (SIGINT) 和 SIGTERM 信号 (Unix)");
                // 创建一个 SIGTERM 信号的监听流
                let mut sigterm = signal(SignalKind::terminate())
                    .expect("无法安装 SIGTERM 信号处理器");

                tokio::select! {
                    _ = signal::ctrl_c() => {
                        info!("接收到 Ctrl+C (SIGINT) 信号，开始优雅停机...");
                    },
                    _ = sigterm.recv() => {
                        info!("接收到 SIGTERM 信号 (来自 K8s/Docker)，开始优雅停机...");
                    },
                }
            } else {
                // 否则 (比如在纯 Windows 上编译)，只监听 Ctrl+C
                info!("正在监听 Ctrl+C (SIGINT) 信号 (Non-Unix)");
                signal::ctrl_c().await
                    .expect("无法安装 Ctrl+C 信号处理器");
                info!("接收到 Ctrl+C (SIGINT) 信号，开始优雅停机...");
            }
        }

        // --- 停机逻辑保持不变 ---

        // 1. 主动从 Nacos 注销服务
        info!("正在从 Nacos 注销服务...");
        if let Err(e) = setup::deregister_nacos_instance(&shutdown_config, &shutdown_naming_client).await {
            error!("从 Nacos 注销服务失败: {}", e);
        } else {
            info!("已成功从 Nacos 注销");
        }
        
        info!("Axum 服务正在关闭...");
    };


    // 6. 启动服务器 (逻辑仍在 setup 模块)
    info!("服务器即将启动在: {}", &config.server_addr);
    setup::run_server(app, &config.server_addr, shutdown_signal).await?;

    Ok(())
}
















