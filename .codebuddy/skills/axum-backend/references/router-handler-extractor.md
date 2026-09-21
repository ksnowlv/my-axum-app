# Router / Handler / Extractor 约定

## 路由

```rust
use axum::{Router, routing::{get, post, patch, delete}};

Router::new()
    .route("/users", get(list).post(create))
    .route("/users/{id}", get(detail).patch(update).delete(remove))
    .route("/users/{id}/orders", get(list_orders))
```

- 同一路径多个方法链式调用；不同路径各写一条 `.route`。
- 冲突路径（`/{id}` 与 `/me`）必须把静态段写在动态段之前。
- 404 fallback 用 `.fallback(handler_not_found)`，返回 `ApiResponse` 格式。
- 需要嵌套时优先 `nest("/prefix", sub_router())`；`Router::merge` 仅用于合并同层路由。

## Handler 签名

```rust
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> AppResult<ApiResponse<Page<User>>> {
    let page = service::list_users(&state.pool, q).await?;
    Ok(ApiResponse::ok(page))
}

pub async fn create(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> AppResult<(StatusCode, ApiResponse<User>)> {
    let user = service::create_user(&state.pool, payload).await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(user)))
}
```

顺序规则：axum 要求**所有** extractor 中除最后一个外都必须实现 `FromRequestParts`。`State`、`Query`、`Path`、`TypedHeader`、`Extension` 属于 parts 类，可放在前面；`Json`、`Bytes`、`String`、`Form` 等消耗 body 的 extractor 只能放最后，且**每个 handler 最多一个**。

返回类型规则：

- 统一 `Result<T, AppError>`，即 `AppResult<T>`。
- 业务数据一律包 `ApiResponse<T>`。
- 需要非 200 状态码时返回 `(StatusCode, ApiResponse<T>)`。

## 路径与查询参数

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct ListQuery {
    #[validate(range(min = 1))]
    pub page: Option<i64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<i64>,
    pub keyword: Option<String>,
}
```

- `Path` 支持元组解构：`Path<(Uuid, Uuid)>` 对应 `/a/{x}/b/{y}`。
- 可选查询参数用 `Option<T>`；必填用 `T`，缺失时 axum 自动 400。
- 分页参数统一 `page` / `page_size`，并提供默认值常量。

## 自定义 Extractor

解析认证信息时实现 `FromRequestParts`，避免消耗 body：

```rust
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use crate::error::AppError;

pub struct CurrentUser(pub Uuid);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Uuid>()
            .copied()
            .map(CurrentUser)
            .ok_or(AppError::Unauthorized)
    }
}
```

校验型 extractor（复用 `validator`）：

```rust
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate().map_err(|e| AppError::Validation(e.to_string()))?;
        Ok(ValidatedJson(value))
    }
}
```

约定：

- 只在 `extractor.rs` 或 `error.rs` 邻近位置定义通用 extractor（本项目放在 `src/extractor.rs`）。
- 自定义 extractor 的 `Rejection` 统一为 `AppError`，保证错误响应格式一致。
- 需要读 body 又需要校验时，用上述 `ValidatedJson`；不要在 handler 内手动调 `validate()`。

## State

- 通过 `State<AppState>` 注入；子路由 `Router<AppState>` 在根路由 `.with_state()` 前保持未注入状态。
- 禁止在 handler 中构造新的连接池或读取配置文件。
- 需要只读配置时用 `state.settings`，不要引入全局 `OnceLock<Settings>`。

## 常见陷阱

- `Router<AppState>` 与 `Router`（已注入状态）是不同类型，`merge` 时类型必须一致。
- `into_make_service()` 只在 `axum::serve` 之前的旧写法需要；本项目的 `axum::serve` 直接接受 `Router`。
- 闭包 handler 只用于探针类极简响应，业务 handler 一律写成具名 `async fn`，便于 `#[utoipa::path]` 与单测引用。
