# Chat - 实时聊天系统后端

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8-brightgreen)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15-blue)](https://www.postgresql.org/)

## 一、项目简介

本项目是基于 Rust + Axum + Tokio 开发的高性能异步 REST 后端即时通讯服务，对标传统 Java/Go 后端，主打低内存占用、高并发吞吐、内存安全无野指针。
实现用户鉴权、工作空间隔离、聊天管理、消息收发、文件上传、实时推送等企业级能力。
采用工程化规范设计，适合高并发业务场景落地。

## 二、技术栈选型

- **异步运行时**：Tokio，高性能异步调度，支撑高并发 IO
- **Web 框架**：Axum 0.8，基于 Tower 生态，轻量高性能、中间件易扩展
- **数据库驱动**：SQLx 异步驱动，无 ORM 开销，原生手写 SQL 可控
- **实时通信**：PostgreSQL LISTEN/NOTIFY 订阅机制 + SSE Server-Sent Events 轻量推送
- **认证授权**：JWT 非对称加密（Ed25519） + Argon2 密码哈希
- **API 文档**：Utoipa + Swagger-UI，自动化接口文档
- **工程化**：Cargo Workspace 工作空间、pre-commit 代码检查、CI/CD 自动化

## 三、核心功能模块

### 用户层

- JWT 登录注册、Token 验证中间件、双通道认证（Header/Query）
- 密码 Argon2 加密存储

### 接口层

- RESTful 规范、OpenAPI 自动化文档
- 统一响应封装、参数校验、utoipa 注解

### 业务层

- 工作空间（Workspace）多租户隔离
- 聊天会话 CRUD、消息收发、文件上传下载
- 实时消息推送（基于 PostgreSQL 触发器 + NOTIFY）

### 数据层

- PostgreSQL 异步 CRUD、事务保障
- 数据库触发器实现消息变更通知

### 基础层

- 结构化日志（tracing）
- 全局错误处理、统一 JSON 返回
- 配置中心化（YAML）

## 四、架构设计

```text
┌─────────────────────────────────────────────────────────────┐
│                      客户端请求                              │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Axum 路由层                                │
│  /api/signup  /api/signin  /api/chats  /api/upload          │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                 中间件层 (Middleware)                          │
│  Token 验证 → 日志追踪 → 请求 ID 注入                         │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Handlers 业务层                             │
│  auth │ chat │ message │ workspace │ file                   │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Models 数据层                              │
│  SQLx 异步查询 → 连接池管理                                   │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  PostgreSQL 数据库                            │
│  数据存储 │ 触发器 → NOTIFY → 实时推送                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                  Notify Server (SSE 独立服务)                  │
│  LISTEN 监听 → 广播到在线用户                                 │
└─────────────────────────────────────────────────────────────┘
```

## 五、关键技术难点 & 解决方案

### 1. 多租户数据隔离

**问题**：如何在同一套 API 下安全隔离不同 Workspace 的数据？

**解决**：数据库表设计携带 `ws_id` 外键，查询时强制携带 Workspace 条件，用户Token 中嵌入 ws_id，查询自动过滤。

### 2. 实时消息推送架构

**问题**：如何在不引入复杂消息队列的情况下实现实时通知？

**解决**：利用 PostgreSQL 触发器监听数据变更，触发 `pg_notify` 通知，独立 Notify Server 订阅并通过 SSE 推送给前端。

## 六、快速部署 & 运行方式

### 编译运行

```bash
# 克隆项目
cargo build --release

# 启动主服务
cd chat-server
cargo run --release

# 启动通知服务（可选）
cd notify-server
cargo run --release
```

### 测试

见 `chat-server/test.rest`

## 可复用模块

- `chat-core/src/utils/jwt.rs` - JWT Ed25519 签名/验签
- `chat-core/src/middlewares/auth.rs` - Token 验证中间件
- `chat-server/src/lib.rs` - AppState 状态管理模式
- Workspace 多租户隔离设计

## 许可证

MIT License
