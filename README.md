# Axum 生产级项目模板 (`axum-template`)

这是一个基于 `axum` (v0.8) 的、结构化的、生产级的 Rust Web 服务模板。它集成了 Nacos 作为服务发现和动态配置中心，并使用 SeaORM (v1.0) 作为数据库 ORM，以及 `bb8-redis` 作为 Redis 连接池。

本项目旨在提供一个高度模块化、可扩展且遵循 Rust 最佳实践的后端服务起点，其架构设计深受 Spring Boot 思想启发，但以 Rust 的方式（显式、安全、高性能）实现。

## 核心技术栈

- **Web 框架:** `axum` (v0.8)
- **异步运行时:** `tokio` (v1)
- **服务治理:** `nacos-sdk` (v0.5) - 用于服务发现和动态配置
- **数据库 (ORM):** `sea-orm` (v1.0)
- **数据库 (Driver):** `sqlx-mysql`
- **Redis 客户端:** `redis` (v0.32) + `bb8-redis` (v0.24) (连接池)
- **HTTP 客户端:** `reqwest` (v0.12) - 用于服务间调用
- **序列化/反序列化:** `serde` (serde_yaml, serde_json)
- **错误处理:** `anyhow` (用于 `main`), `thiserror` (用于 `AppError`)
- **日志:** `tracing` (集成 `tower-http`)
- **异步 Trait:** `async-trait` (用于 Nacos 监听器)
- **条件编译:** `cfg-if` (用于优雅停机)
- **配置占位符:** `shellexpand` (已移除，采用 Nacos 直配)

## 架构亮点

- **清晰的分层 (SoC):**
  - `main.rs`: 应用程序入口 (极其简洁，只负责调用)。
  - `router.rs`: 封装所有 Axum 路由和顶层中间件的组装。
  - `setup/`: 封装所有“一次性”的启动任务（如创建客户端、连接池、添加监听器）。
  - `state.rs`: 定义全局共享状态 (`AppState`)。
  - `config/`: 定义和加载配置结构。
  - `handlers/`: HTTP 处理器 (Controllers)，负责请求校验、DTO 转换和调用 Service。
  - `services/`: 核心业务逻辑，不关心 HTTP。
  - `repository/`: 数据库访问层，封装 `SeaORM` 查询。
  - `models/`: `SeaORM` 实体 (Entities) 定义。
  - `clients/`: 封装对外部微服务（如 `auth-service`）的 HTTP 调用 (类似 Feign)。
  - `response.rs`: 统一的 API 响应结构 `ApiResponse<T>` (类似 Java 的 `R<T>`)。
  - `errors.rs`: 统一的错误处理 (`AppError` + `ServiceError` + `impl IntoResponse`)。
- **服务发现:** 启动时自动将服务注册到 Nacos。
- **动态配置:** 启动时从 Nacos 加载配置，并**实时监听**配置变更，通过 `RwLock` 动态更新 `AppState` 中的配置。
- **认证与授权:**
  - `middleware/auth.rs`:
    - 实现了“自认证”中间件 `mw_require_auth`，通过 `clients/auth_client` 调用 `auth-service` 的 `/check_token` 接口，并提取 `CurrentUser` 放入请求 `extensions` 中。
    - **提供了 `check_permission` 辅助函数**，用于在 Handler 内部检查 `CurrentUser` 的权限。
  - `handlers/*.rs`:
    - 受保护的 Handler (处理器) 负责在函数开头通过 `Extension<Arc<CurrentUser>>` 获取用户。
    - 并**在函数体内部第一行**调用 `check_permission(&user, "...")?` 来实现类似 `@PreAuthorize` 的声明式权限检查。
- **优雅停机 (Graceful Shutdown):**
  - 同时监听 `Ctrl+C` (本地开发) 和 `SIGTERM` (K8s/Docker)。
  - 在服务停止前，**主动**向 Nacos 注销服务实例，防止流量黑洞。

## 目录结构

```
axum-template/
├── src/
│   ├── main.rs         # 应用程序入口
│   ├── router.rs       # 路由组装
│   ├── state.rs        # AppState 定义
│   ├── errors.rs       # AppError / ServiceError / IntoResponse
│   ├── response.rs     # ApiResponse<T> 定义
│   │
│   ├── config/         # 配置
│   │   ├── mod.rs      # 基础配置 (Config, 从 .env 加载)
│   │   └── app_specific.rs # 业务配置 (AppSpecificConfig, 从 Nacos 解析)
│   │
│   ├── setup/          # 启动逻辑封装
│   │   ├── mod.rs      # 顶层 setup 函数 (setup_application_state, run_server)
│   │   ├── database.rs # build_db_pool
│   │   ├── http.rs     # build_http_client
│   │   ├── nacos.rs    # Nacos 客户端/注册/监听器
│   │   └── redis.rs    # build_redis_pool
│   │
│   ├── clients/        # 微服务客户端 (类似 Feign)
│   │   ├── mod.rs      # 声明
│   │   ├── service_client.rs # 通用 Nacos HTTP 客户端
│   │   └── auth_client.rs    # Auth 服务客户端
│   │
│   ├── handlers/       # HTTP 处理器 (Controllers)
│   │   ├── mod.rs
│   │   ├── health_handler.rs
│   │   ├── hello_handler.rs
│   │   ├── kms_app_access_handler.rs
│   │   └── redis_handler.rs
│   │
│   ├── middleware/     # 中间件
│   │   ├── mod.rs
│   │   ├── auth.rs     # 认证 (mw_require_auth, CurrentUser) 和授权 (check_permission)
│   │   └── logging.rs
│   │
│   ├── models/         # 数据库实体 (SeaORM)
│   │   ├── mod.rs
│   │   └── kms_app_access.rs
│   │
│   ├── repository/     # 数据库访问 (SeaORM)
│   │   ├── mod.rs
│   │   └── kms_app_access_repo.rs
│   │
│   └── services/       # 业务逻辑
│       ├── mod.rs
│       └── kms_app_access_service.rs
│
├── tests/              # 集成测试
│   └── health_check.rs
│
├── .env                # 本地开发环境变量
├── Cargo.toml          # 依赖管理 (pom.xml)
├── Dockerfile          # 多阶段、可重用的 Dockerfile
├── .dockerignore       # Docker 忽略文件
├── .gitignore          # Git 忽略文件
└── README.md           # 本文档
```

## 5. 微服务集成 (与 Spring Cloud 融合)

本模板被设计为可以无缝（或通过最小改动）融入现有的 Spring Cloud + Nacos 体系。

### Java (Spring Boot) 调用 Rust (Axum)

Spring Boot 服务可以通过 Nacos 和 OpenFeign 像调用其他 Java 服务一样调用本服务。

1. **Rust 服务注册:** 本模板 (`axum-template`) 启动时会以 `APP_NAME` (来自 `.env`，例如 `axum-template-service`) 注册到 Nacos。

2. **Java Feign 客户端:** 在你的 Spring Boot 项目中，定义一个 Feign 客户端：

   ```
   // 1. `value` 必须匹配 Rust 服务的 APP_NAME
   @FeignClient(value = "axum-template-service")
   public interface RemoteRustService {
   
       // 2. 路由和响应体必须匹配 Rust 端的定义
   
       // 对应 /hello 路由
       @GetMapping("/hello")
       ApiResponse<HelloResponseDto> getHelloConfig();
   
       // 对应 /app-access/{id} 路由
       @GetMapping("/app-access/{id}")
       ApiResponse<KmsAppAccessResponseDto> getAppAccessById(@PathVariable("id") Long id);
   
       // (注意：你需要在 Java 端定义 ApiResponse<T>, HelloResponseDto, KmsAppAccessResponseDto)
       // (它们必须与 Rust 的 `response.rs` 和 `handlers/` 里的 DTO 结构一致)
   }
   ```

### Rust (Axum) 调用 Java (Spring Boot)

本模板通过 `src/clients/` 模块实现了“Rust 版的 Feign”，它封装了服务发现和 HTTP 调用。

1. **Java 服务注册:** 你的 Java 服务（例如 `upms-service`）正常注册到 Nacos。
2. **Rust 客户端封装:**
   - `src/clients/service_client.rs`: 提供了通用的 `get_service` 和 `post_service` 函数。它们封装了 Nacos 服务发现 (`select_one_healthy_instance`) 和 `reqwest` HTTP 调用。
   - `src/clients/auth_client.rs`: 提供了**特定业务**的客户端。它负责：
     - 从 `Config` 中获取服务名 (e.g., `AUTH_SERVICE_NAME`)。
     - 准备请求参数 (`query_params`)。
     - 调用 `service_client::get_service_with_group(...)`。
     - 将返回的 JSON 响应解析为 Rust `struct`。
3. **调用**:
   - `src/middleware/auth.rs`（认证中间件）通过 `State<AppState>` 获取共享状态，然后调用 `auth_client::check_token(&state, ...)` 来执行服务间调用。

## 6. 运行步骤 (精简版)

1. **准备环境:** Nacos, MySQL, Redis, `auth-service` (Java) 正在运行。

2. **准备数据库:** 在 MySQL 中创建库和 `kms_app_access` 表。

3. **配置 Nacos:** 在**非 public** 命名空间下，创建 `axum-template.yaml` 配置（见上方 YAML 示例）。

4. **配置本地 `.env`:**

   - 复制 `README.md` 中的 `.env` 示例，创建 `.env` 文件。
   - **必须**修改 `NACOS_NAMING_NAMESPACE`, `NACOS_CONFIG_NAMESPACE`, `NACOS_ADDR`。
   - **必须**修改 Nacos 中 `database.url` 的用户名/密码。

5. **运行:**

   ```
   cargo run
   ```

6. **测试:**

   - `curl http://localhost:4000/` (健康检查)
   - `curl -H "Authorization: Bearer <token>" http://localhost:4000/hello`
   - `curl -H "Authorization: Bearer <token>" http://localhost:4000/app-access/1`

7. **构建生产镜像:**

   ```
   docker build --build-arg APP_NAME=axum-template -t my-axum-app .
   # 运行时 *必须* 通过 -e 传入所有 .env 变量
   docker run -p 4000:4000 -e APP_NAME="axum-template" -e SERVER_ADDR="0.0.0.0:4000" -e NACOS_ADDR="..." ... my-axum-app
   ```
