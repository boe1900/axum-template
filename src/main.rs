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

// 更新 use 语句以指向新的模块路径
use crate::config::app_specific::{parse_nacos_config, AppSpecificConfig};
use crate::config::Config;
use crate::state::AppState;
// 修改后的 use 语句，移除了 ServiceExt
use axum::{Router, middleware as axum_middleware, routing::get};
use nacos_sdk::api::config::{ConfigChangeListener, ConfigResponse, ConfigService, ConfigServiceBuilder};
use nacos_sdk::api::naming::{NamingService, NamingServiceBuilder, ServiceInstance};
use nacos_sdk::api::props::ClientProps;

use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info}; // 保持导入 error 宏，监听器可能还需要
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

    // 3. 创建 Nacos 客户端
    info!("正在连接 Nacos: {}", &config.nacos_addr);
    let naming_client = Arc::new(build_nacos_naming_client(&config)?);
    let config_client = Arc::new(build_nacos_config_client(&config)?);

    // 4. 获取初始 Nacos 配置并解析
    let initial_config_content = config_client
        .get_config(config.nacos_config_data_id.clone(), config.nacos_config_group.clone())
        .await?;
    info!("从 Nacos 获取到初始配置字符串: (内容可能较长，后续解析)");

    // 使用 .expect() 替换 match，如果解析失败则直接 panic
    let initial_app_config = parse_nacos_config(&initial_config_content.content())
        .expect("无法解析初始 Nacos 配置！请检查 Nacos 中的配置格式。");
    info!("成功解析初始 Nacos 配置: {:?}", initial_app_config);


    // 将解析后的配置放入 RwLock
    let app_config_rwlock = Arc::new(RwLock::new(initial_app_config));

    // 5. 创建共享的应用状态
    let app_state = AppState {
        naming_client: naming_client.clone(),
        config_client: config_client.clone(),
        app_config: app_config_rwlock.clone(),
    };

    // 6. 添加配置监听器 (将 RwLock 传递给它)
    config_client
        .add_listener(
            config.nacos_config_data_id.clone(),
            config.nacos_config_group.clone(),
            Arc::new(AppConfigChangeListener {
                app_config: app_config_rwlock,
            }),
        )
        .await?;
    info!("已添加 Nacos 配置监听器");

    // 7. 注册服务实例到 Nacos
    register_nacos_instance(&config, &app_state.naming_client).await?;

    // 8. 定义和组装 Axum 路由
    // 调整了 .with_state() 的顺序：放在路由定义之后，中间件之前
    let app: Router = Router::new() // 不再需要 AppState 注解
        // 添加路由
        .route("/", get(handlers::health_check))
        .nest("/hello", handlers::hello_handler::routes())
        // (将来可以取消注释)
        // .nest("/users", handlers::user_handler::routes())

        // 应用状态 (在路由之后，中间件之前)
        .with_state(app_state) // <--- 移到了这里

        // 应用中间件 (在状态之后)
        .layer(axum_middleware::from_fn(
            middleware::logging::log_requests,
        ));


    // 9. 启动服务器
    let listener = TcpListener::bind(&config.server_addr).await?;
    info!("Server listening on {}", &config.server_addr);
    // 确保调用的是 app.into_make_service()，并且 ServiceExt trait 已导入
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

// --- 辅助函数 ---

/// 根据 Config 构建 Nacos ClientProps
fn build_nacos_client_props(config: &Config) -> ClientProps {
    let mut props = ClientProps::new()
        .server_addr(config.nacos_addr.clone())
        .namespace(config.nacos_namespace.clone())
        .app_name("axum-template-service"); // 服务名硬编码，也可放入配置
    // 只有当用户名和密码都存在时，才设置认证信息
    if let (Some(username), Some(password)) = (config.nacos_username.clone(), config.nacos_password.clone()) {
        props = props.auth_username(username).auth_password(password);
    }
    props
}

/// 构建 Nacos Naming (服务发现) 客户端
// 接收 &Config 而不是 ClientProps，以便检查用户名
fn build_nacos_naming_client(config: &Config) -> anyhow::Result<NamingService> {
    let props = build_nacos_client_props(config); // 先构建基础 props
    let builder = NamingServiceBuilder::new(props);

    // 直接检查 config 中的 username 是否存在
    let client = if config.nacos_username.is_some() {
        // 如果有用户名（假设密码也必然有），启用 auth_plugin_http
        builder.enable_auth_plugin_http().build()?
    } else {
        builder.build()?
    };
    Ok(client)
}

/// 构建 Nacos Config (配置中心) 客户端
// 接收 &Config 而不是 ClientProps
fn build_nacos_config_client(config: &Config) -> anyhow::Result<ConfigService> {
    let props = build_nacos_client_props(config); // 先构建基础 props
    let builder = ConfigServiceBuilder::new(props);

    // 直接检查 config 中的 username 是否存在
    let client = if config.nacos_username.is_some() {
        // 如果有用户名，启用 auth_plugin_http
        builder.enable_auth_plugin_http().build()?
    } else {
        builder.build()?
    };
    Ok(client)
}


/// 注册服务实例到 Nacos
async fn register_nacos_instance(config: &Config, client: &Arc<NamingService>) -> anyhow::Result<()> {
    // 从 server_addr (例如 "127.0.0.1:3000") 中解析出 IP 和 Port
    let parts: Vec<&str> = config.server_addr.split(':').collect();
    let ip = parts.get(0).unwrap_or(&"127.0.0.1").to_string(); // 提供默认 IP
    let port: i32 = parts.get(1).unwrap_or(&"3000").parse()?; // 提供默认端口并解析

    let service_name = "axum-template-service".to_string(); // 服务名硬编码

    // 创建 ServiceInstance
    let instance = ServiceInstance {
        ip: ip.clone(),
        port,
        ..Default::default() // 使用其他字段的默认值
    };

    // 调用 Nacos SDK 注册
    // 假设不需要 group_name，传 None
    client.register_instance(service_name.clone(), None, instance).await?;

    info!(
        "服务已成功注册到 Nacos: {} at {}:{}",
        service_name, ip, port
    );
    Ok(())
}


// --- Nacos 配置监听器实现 ---
// (监听器中的错误处理保持不变，因为它是在运行时发生，不应让整个服务崩溃)
struct AppConfigChangeListener {
    app_config: Arc<RwLock<AppSpecificConfig>>,
}

// --- 修改点 ---
// 移除 #[async_trait::async_trait]
// 将 fn notify 改为同步函数
impl ConfigChangeListener for AppConfigChangeListener {
    fn notify(&self, config_resp: ConfigResponse) {
        // 使用 getter 方法 .data_id() 和 .group() 访问私有字段
        info!("[Nacos Listener] 配置发生变更，准备更新: Data ID={}, Group={}", config_resp.data_id(), config_resp.group());

        // 克隆 Arc 指针，以便在异步任务中使用
        let app_config_clone = self.app_config.clone();
        // 克隆配置内容，因为 config_resp 生命周期可能不够长
        let content_clone = config_resp.content().to_string(); 

        // --- 修改点 ---
        // 使用 tokio::spawn 在 Tokio 运行时中异步执行更新逻辑
        tokio::spawn(async move {
            match parse_nacos_config(&content_clone) {
                Ok(new_config) => {
                    info!("成功解析 Nacos 配置变更: {:?}", new_config);
                    // 在异步任务中获取写锁
                    let mut config_guard = app_config_clone.write().await;
                    *config_guard = new_config;
                    info!("AppState 中的配置已更新");
                }
                Err(e) => {
                    // 在运行时解析失败，只记录错误，不崩溃
                    error!("解析 Nacos 配置变更失败: {}", e);
                }
            }
        });
    }
}


