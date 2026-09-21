# 架构骨架与基础代码

## Cargo.toml 依赖基线

```toml
[package]
name = "my-axum-app"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors", "timeout", "compression-gzip", "request-id", "set-header"] }

serde = { version = "1", features = ["derive"] }
serde_json = "1"
validator = { version = "0.19", features = ["derive"] }

chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }

sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid", "migrate", "macros"] }

tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

config = "0.15"
dotenvy = "0.15"
thiserror = "2"

utoipa = { version = "5", features = ["chrono", "uuid"] }
# vendored 必须开启：否则构建时会从 GitHub 下载 swagger-ui 静态资源，离线环境会失败
utoipa-swagger-ui = { version = "9", features = ["axum", "vendored"] }

[dev-dependencies]
axum-test = "17"
serial_test = "3"
```

> `sqlx` 启用 `macros` 后编译需 `DATABASE_URL` 或 `.sqlx` 离线缓存；CI 前执行 `cargo sqlx prepare`。

项目同时存在 `src/main.rs` 与 `src/bin/openapi.rs`（文档导出）等多个 bin，必须在 `Cargo.toml` 中声明默认入口，否则 `cargo run` 与热重载的 `cargo watch -x run` 都会报 “could not determine which binary to run”：

```toml
[package]
default-run = "my-axum-app"

[lib]
name = "my_axum_app"
path = "src/lib.rs"
```

## main.rs：只做四件事

```rust
mod config;
mod error;
mod features;
mod logging;
mod response;
mod router;
mod state;

use config::Settings;
use sqlx::postgres::PgPoolOptions;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load()?;              // 1. 配置
    logging::init(&settings.log_level);            // 2. 日志

    let pool = PgPoolOptions::new()                // 3. 连接池
        .max_connections(settings.database.max_connections)
        .connect(&settings.database.url)
        .await?;
    sqlx::migrate!().run(&pool).await?;

    let state = AppState::new(pool, settings.clone());
    let app = router::build(state.clone());        // 4. 路由 + 中间件

    let listener = tokio::net::TcpListener::bind(&settings.server.addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("shutdown signal received");
}
```

## state.rs：唯一的状态载体

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub settings: std::sync::Arc<crate::config::Settings>,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool, settings: crate::config::Settings) -> Self {
        Self { pool, settings: std::sync::Arc::new(settings) }
    }
}
```

- `AppState` 必须 `Clone`（内部全部 `Arc`/`Pool`），因为 `Router::with_state` 会为每个连接克隆。
- 新增全局依赖（Redis、HTTP client、消息队列）一律作为 `AppState` 字段，禁止全局静态变量。

## router.rs：组装根路由与中间件

```rust
use axum::Router;
use crate::{features, state::AppState};

pub fn build(state: AppState) -> Router {
    let api = Router::new()
        .nest("/users", features::user::router())
        .nest("/orders", features::order::router());

    Router::new()
        .route("/health", axum::routing::get(health))
        .nest("/api/v1", api)
        .merge(crate::docs::router())   // utoipa swagger-ui
        .with_state(state)
        .layer(tower::ServiceBuilder::new().map_request(/* ... */)) // 见 middleware.md
}

async fn health() -> &'static str { "ok" }
```

要点：

- 先 `.nest` / `.route` 组装，最后统一 `.with_state(state)`；类型由 `Router<AppState>` 收敛为 `Router`。
- 中间件栈集中在 `router.rs`，业务域 `mod.rs` 内不要再叠加全局中间件。

## features/<feature>/mod.rs：域内路由

```rust
pub mod handler;
pub mod model;
pub mod repository;
pub mod service;

use axum::{Router, routing::get};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(handler::list).post(handler::create))
        .route("/users/{id}", get(handler::detail).patch(handler::update).delete(handler::remove))
}
```

命名与挂载规则：

- `router()` 返回 `Router<AppState>`，由 `router.rs` 统一 `nest` 到 `/api/v1/<复数资源>`。
- 路径参数用 `{id}`，handler 中用 `Path<Uuid>` 或 `Path<i64>` 提取。
- 跨域复用（如 `/users/{id}/orders`）在父域 `router()` 内直接声明，不要嵌套子 Router。

## 分层检查清单

- [ ] `handler.rs` 中不出现 SQL、不出现 `PgPool` 直接字段访问。
- [ ] `repository.rs` 之外没有 `sqlx::query*` 调用。
- [ ] `service.rs` 的公开函数返回 `AppResult<T>`，不返回 HTTP 类型（`Response`/`StatusCode`）。
- [ ] `model.rs` 中的 DTO 已 `#[derive(Validate, ToSchema)]`，实体已 `#[derive(FromRow, Serialize, ToSchema)]`。
- [ ] 所有新增模块已在 `src/features/mod.rs` 声明并在 `src/router.rs` 挂载。
