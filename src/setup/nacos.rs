
use crate::config::{
    app_specific::{parse_nacos_config, AppSpecificConfig}, // <-- 新增
    Config,
};
use nacos_sdk::api::{
    config::{ConfigChangeListener, ConfigResponse, ConfigService, ConfigServiceBuilder}, // <-- 新增
    naming::{NamingService, NamingServiceBuilder, ServiceInstance},
    props::ClientProps,
};
use std::sync::Arc; 
use tokio::sync::RwLock;
use tracing::{error, info};


pub fn build_nacos_client_props(config: &Config) -> ClientProps {
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
pub fn build_nacos_naming_client(config: &Config) -> anyhow::Result<NamingService> {
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
pub fn build_nacos_config_client(config: &Config) -> anyhow::Result<ConfigService> {
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
pub async fn register_nacos_instance(config: &Config, client: &Arc<NamingService>) -> anyhow::Result<()> {
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
pub struct AppConfigChangeListener {
    pub app_config: Arc<RwLock<AppSpecificConfig>>
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