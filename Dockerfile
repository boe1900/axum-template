# --- 阶段 1: 构建 (Builder) ---
FROM rust:1-slim-bookworm as builder

# ARG APP_NAME 默认值来自 Cargo.toml
ARG APP_NAME=axum-template

RUN apt-get update && apt-get install -y musl-tools musl-dev
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# --- 更健壮的依赖缓存层 (保持不变) ---
RUN mkdir src
RUN echo 'fn main() { panic!("This is a dummy main for caching dependencies"); }' > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

# 复制真正的源代码
COPY src/ ./src/

# 编译我们自己的代码
RUN cargo build --release --target x86_64-unknown-linux-musl --bin $APP_NAME


# --- 阶段 2: 运行 (Runtime) ---
FROM alpine:latest

ARG APP_NAME=axum-template
# --- 新增：为端口号也添加 ARG ---
# 默认值应与 .env.example 或 config.rs 中的默认值一致
ARG APP_PORT=4000

# 创建用户 (保持不变)
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# 复制编译好的可执行文件 (保持不变)
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/$APP_NAME /usr/local/bin/app

# 切换到非 root 用户
USER appuser

# --- 修改点：使用 $APP_PORT ---
# 暴露我们在 ARG 中定义的端口
EXPOSE $APP_PORT

# --- 修改点：使用 $APP_PORT ---
ENV RUST_LOG="info,${APP_NAME}=debug,nacos_sdk=info"
# 使用 $APP_PORT 来设置默认的 SERVER_ADDR
ENV SERVER_ADDR="0.0.0.0:${APP_PORT}"

# 运行程序 (保持不变)
ENTRYPOINT ["/usr/local/bin/app"]

