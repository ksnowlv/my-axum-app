---
name: axum-backend
description: my-axum-app（Rust + Axum 0.8 + sqlx/PostgreSQL）后端开发规范与脚手架。本 skill 应在新增或修改路由、Handler、Extractor、中间件、tracing 日志、sqlx 数据访问、统一错误与响应封装、配置加载，以及编写集成测试或 utoipa OpenAPI 文档时使用；也应在任何触及 src/ 目录结构、分层职责或代码风格一致性的任务中使用。
---

# my-axum-app 后端开发规范

本 skill 固化 `my-axum-app` 的技术栈、目录结构、分层职责与编码约定，使后续新增功能时无需重复推导架构决策。

## 技术栈基线

| 领域 | 选型 | 备注 |
| --- | --- | --- |
| Web 框架 | `axum` 0.8 | 配合 `tokio` 1（features = full） |
| 异步运行时 | `tokio` | `#[tokio::main]` |
| 数据库 | PostgreSQL + `sqlx` 0.8 | 编译期校验 SQL，启用 `postgres/runtime-tokio-rustls/chrono/uuid/migrate` |
| 序列化 | `serde` / `serde_json` | 统一 snake_case |
| 校验 | `validator` | DTO 层声明式校验 |
| 日志 | `tracing` + `tracing-subscriber` | 禁用 `println!` |
| 中间件 | `tower` + `tower-http` | `TraceLayer`、`CorsLayer`、`TimeoutLayer` 等 |
| 配置 | `config` + `dotenvy` | 分层加载：`config/default.toml` + `config/{env}.toml` + 环境变量 |
| API 文档 | `utoipa` (+ `utoipa-swagger-ui`) | `#[utoipa::path]` 与代码同处维护 |
| 时间/ID | `chrono`、`uuid` | 时间统一 UTC（`DateTime<Utc>`） |

Rust edition 为 2024。新增依赖时保持最小可用，避免引入与上述能力重复的 crate。

## 参考文档索引

按需加载，不要一次性全部读入：

| 任务 | 加载文件 |
| --- | --- |
| 搭建/调整目录骨架、分层职责、Cargo.toml | `references/architecture.md` |
| 新增或修改路由、Handler、State、Extractor | `references/router-handler-extractor.md` |
| 自定义错误、统一响应包装、校验失败处理 | `references/error-and-response.md` |
| 连接池、迁移、查询、事务、测试数据库 | `references/database-sqlx.md` |
| 认证、请求 ID、日志中间件、限流、CORS | `references/middleware.md` |
| tracing 初始化、span、字段、敏感信息脱敏 | `references/logging-tracing.md` |
| 配置项、环境变量、多环境加载 | `references/config-env.md` |
| 集成测试、OpenAPI 文档生成 | `references/testing-openapi.md` |
| 本地热重载、调试技巧、常见故障 | `references/dev-workflow.md` |

## 目录结构（强制）

```text
src/
├── main.rs              # 仅做：加载配置 → 初始化日志 → 建池 → serve
├── lib.rs               # 对外暴露 App/router，供集成测试复用
├── config.rs            # Settings 定义与加载
├── logging.rs           # tracing 初始化
├── error.rs             # AppError / AppResult
├── response.rs          # ApiResponse<T>
├── state.rs             # AppState
├── router.rs            # 根路由组装 + 中间件栈
└── features/
    ├── mod.rs
    └── <feature>/       # 业务域，如 user / order
        ├── mod.rs        # 该域的 Router 组装
        ├── handler.rs    # HTTP 层：解析入参、调用 service、返回 ApiResponse
        ├── service.rs    # 业务层：业务规则、事务边界
        ├── repository.rs # 数据层：唯一允许写 SQL 的地方
        └── model.rs      # 实体 + DTO + utoipa Schema
migrations/              # sqlx 迁移，文件名 YYYYMMDDHHMMSS_<name>.sql
tests/                   # 集成测试，一个域一个文件
config/                  # default.toml / development.toml / production.toml
```

## 分层职责与依赖方向

```text
router → handler → service → repository → model
```

- `handler`：只做参数提取、调用 service、包装响应。**禁止**写 SQL、禁止 `unwrap()`、禁止直接持有 `PgPool`。
- `service`：业务规则、事务边界、跨 repository 编排。依赖 `&PgPool` 或显式传入的 `&mut Transaction`。
- `repository`：唯一出现 SQL 的位置，函数签名统一 `async fn xxx(pool: &PgPool, ...) -> AppResult<T>`。
- `model`：纯数据结构，不含业务逻辑。
- 反向依赖（如 repository 调用 service）一律禁止。

## 新增业务域的标准流程

1. 生成骨架：`bash .codebuddy/skills/axum-backend/scripts/new_feature.sh <feature_snake_case>`。
2. 编写 `migrations/` 下的建表 SQL，执行 `sqlx migrate run` 并 `cargo sqlx prepare`（保持离线编译可用）。
3. 在 `model.rs` 补齐实体与 DTO，DTO 必须加 `#[derive(Validate)]` 与 `#[derive(ToSchema)]`。
4. 在 `repository.rs` 实现数据访问，所有 SQL 使用 `sqlx::query_as!` / `sqlx::query!` 宏。
5. 在 `service.rs` 实现业务规则，涉及多写操作时用事务。
6. 在 `handler.rs` 实现 Handler，并为每个路由补 `#[utoipa::path]`。
7. 在 `src/features/mod.rs` 注册模块，在 `src/router.rs` 通过 `nest` 挂载。
8. 在 `tests/<feature>.rs` 补集成测试。
9. 运行 `bash .codebuddy/skills/axum-backend/scripts/check.sh` 自检。

## 硬性约定（每次改动都必须满足）

- 错误处理：业务错误统一返回 `AppError`；禁止在 `handler` 中出现 `unwrap()` / `expect()` / `panic!`。
- 响应格式：所有业务接口返回 `ApiResponse<T>`，即 `{ code, message, data }`；`/health` 等探针接口可例外。
- 状态注入：全局依赖只通过 `State<AppState>` 传递，禁止全局单例与 `lazy_static` 数据库连接。
- 日志：统一用 `tracing`（`info!/warn!/error!/debug!` 与 `#[instrument]`），禁止 `println!`。
- 时间：一律 UTC，对外输出 RFC3339；ID 使用 `uuid::Uuid`。
- 命名：模块与字段 snake_case，类型 PascalCase，路由复数小写（`/api/v1/users`）。
- 路由前缀：所有业务路由挂在 `/api/v1` 下。
- 校验：入参校验在 DTO 上声明，在进入 handler 前通过 `ValidatedJson` extractor 完成。
- 测试：新增路由至少补一个集成测试；修改 SQL 后必须同步更新 `.sqlx` 离线缓存。
- 提交前自检：`cargo fmt --all` → `cargo clippy --all-targets -- -D warnings` → `cargo test`。

## 脚本

- `scripts/new_feature.sh <feature>`：按模板生成 `features/<feature>/` 全套文件与迁移 SQL 骨架。
- `scripts/check.sh`：串行执行 fmt / clippy / test 的提交前自检。

项目根目录下的脚本：

- `scripts/dev.sh`：热重载；`.cargo/config.toml` 中的别名 `cargo dev` 是其快捷方式，用法与排障见 `references/dev-workflow.md`
- `scripts/openapi.sh [path]`：生成 OpenAPI 文档文件，默认写入 `openapi.json`
