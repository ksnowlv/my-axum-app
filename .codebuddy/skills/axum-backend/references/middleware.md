# 中间件约定

## 分层结构

中间件分两类，统一在 `src/router.rs` 通过 `tower::ServiceBuilder` 组装：

```rust
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
    timeout::TimeoutLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer, PropagateRequestIdLayer},
};
use axum::http::header::HeaderName;

let middleware_stack = ServiceBuilder::new()
    .layer(SetRequestIdLayer::new(HeaderName::from_static("x-request-id"), MakeRequestUuid))
    .layer(TraceLayer::new_for_http()
        .make_span_with(|req: &axum::extract::Request| {
            use tracing::field::Empty;
            tracing::info_span!("request",
                method = %req.method(), uri = %req.uri(), request_id = Empty, status = Empty)
        })
        .on_response(DefaultOnResponse::new().include_headers(true)))
    .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)))
    .layer(CorsLayer::permissive())   // 生产环境改为显式白名单
    .layer(PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")));

let app = Router::new()
    // ... routes
    .layer(middleware_stack);
```

顺序（从外到内）：RequestId → Trace → 超时 → CORS → 路由 → Handler。

约定：

- 全局中间件只出现在 `router.rs`；域级中间件在域 `router()` 内用 `.layer()` 叠加。
- `TraceLayer` 自带 `x-request-id` 与耗时记录，不要自己再实现一套 access log。
- 生产环境 `CorsLayer` 必须显式指定 `allow_origin` / `allow_methods`，禁止 `permissive()`。

## 函数式中间件

需要访问 `AppState` 时用 `from_fn_with_state`：

```rust
use axum::{middleware::Next, extract::{State, Request}, response::Response};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let user_id = crate::service::auth::verify_token(&state.settings.jwt_secret, token)?;

    req.extensions_mut().insert(user_id);   // 供 CurrentUser extractor 读取
    Ok(next.run(req).await)
}
```

挂载：

```rust
let api = Router::new()
    .nest("/users", features::user::router())
    .layer(axum::middleware::from_fn_with_state(state.clone(), auth_middleware));
```

要点：

- `from_fn` 的闭包/函数第一个参数之后是 `Request`，最后是 `Next`；带 `State` 时 `State` 放最前。
- 中间件返回 `Result<Response, AppError>` 即可复用统一错误响应。
- 中间件中不要读取 body；需要 body 的场景放到 handler 或 extractor。

## 限流

使用 `tower::limit::RateLimitLayer`（按实例计数）或 `governor` + `axum_governor`（按 IP/Key）：

```rust
use tower::limit::RateLimitLayer;
.layer(RateLimitLayer::new(100, std::time::Duration::from_secs(60)))
```

单机场景用 `RateLimitLayer`；多实例部署需改用 Redis 计数。

## 其他常见中间件

| 需求 | 方案 |
| --- | --- |
| 请求体大小限制 | `tower_http::limit::RequestBodyLimitLayer::new(2 * 1024 * 1024)` |
| gzip 压缩 | `tower_http::compression::CompressionLayer::new()` |
| 安全响应头 | `tower_http::set_header::SetResponseHeaderLayer` |
| 静态资源 | `tower_http::services::ServeDir` |
| 优雅关闭 | `axum::serve(...).with_graceful_shutdown(...)` |

## 禁止事项

- 禁止在中间件里做业务数据库写入。
- 禁止在中间件中 `panic!`；失败应返回 `AppError`。
- 禁止把鉴权逻辑复制到每个 handler，统一走中间件 + `CurrentUser` extractor。
