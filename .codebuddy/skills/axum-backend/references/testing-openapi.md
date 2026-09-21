# 集成测试与 OpenAPI 文档

## 测试分层

| 类型 | 位置 | 说明 |
| --- | --- | --- |
| 单元测试 | `src/**` 内 `#[cfg(test)] mod tests` | 纯函数、校验逻辑、解析器 |
| 集成测试 | `tests/<feature>.rs` | 启动真实 App + 测试数据库，走 HTTP |
| SQL 校验 | `cargo sqlx prepare` | 编译期校验查询 |

集成测试必须能独立运行：`cargo test` 前准备 `DATABASE_URL` 指向测试库。

## 测试脚手架

```rust
// tests/common/mod.rs
use my_axum_app::{build_test_app, TestContext};
use axum_test::TestServer;

pub async fn spawn_app() -> (TestServer, TestContext) {
    let ctx = TestContext::new().await;      // 建池 + migrate + 清空表
    let server = TestServer::new(build_test_app(ctx.state.clone())).unwrap();
    (server, ctx)
}
```

`src/lib.rs` 中暴露：

```rust
pub fn build_router(state: AppState) -> Router { router::build(state) }
pub fn build_test_app(state: AppState) -> Router {
    router::build(state).route("/test/seed", axum::routing::post(seed_handler))
}
```

- 测试专用路由仅在 `#[cfg(feature = "test-utils")]` 或 `cfg!(test)` 下编译。
- 使用 `axum_test::TestServer` 免端口绑定；`TestServer::new` 接受 `IntoMakeService`。

## 用例写法

```rust
mod common;

#[tokio::test]
async fn create_user_returns_201() {
    let (server, _ctx) = common::spawn_app().await;

    let resp = server
        .post("/api/v1/users")
        .json(&serde_json::json!({ "email": "a@b.com", "name": "Alice" }))
        .await;

    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["email"], "a@b.com");
}

#[tokio::test]
async fn duplicate_email_returns_409() { /* ... */ }
```

约定：

- 每个新增路由至少覆盖：成功路径、参数校验失败（400）、资源不存在（404）、未授权（401）。
- 数据库副作用测试用事务回滚或 `TRUNCATE` 清理，测试之间用 `serial_test::serial` 串行化。
- 断言统一响应包装的 `code` 字段，而不只是 HTTP 状态码。
- 禁止依赖固定端口与真实外部服务；外部依赖必须 mock 或替换实现。

## OpenAPI（utoipa）

依赖：`utoipa`（derive）、`utoipa-swagger-ui`（axum feature）。

```rust
// src/docs.rs
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(crate::features::user::handler::list, crate::features::user::handler::create),
    components(schemas(
        crate::features::user::model::User,
        crate::features::user::model::CreateUserRequest,
        crate::response::ApiResponse<crate::features::user::model::User>,
    )),
    tags((name = "user", description = "用户接口")),
)]
pub struct ApiDoc;

pub fn router() -> axum::Router<crate::state::AppState> {
    axum::Router::new().merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
```

Handler 标注：

```rust
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "user",
    params(("page" = Option<i64>, Query, description = "页码")),
    responses(
        (status = 200, description = "用户列表", body = ApiResponse<Page<User>>),
        (status = 400, description = "参数校验失败", body = ErrorBody),
    )
)]
pub async fn list(/* ... */) -> AppResult<ApiResponse<Page<User>>> { /* ... */ }
```

要点：

- 每个 handler 都要有 `#[utoipa::path]`；路径写全（含 `/api/v1` 前缀）。
- 所有出现在 `body` 里的类型都要 `#[derive(ToSchema)]` 并注册到 `components(schemas(...))`。
- 分页与错误结构只注册一次，之后复用。
- 新增域时把其 handler 与 schema 追加到 `#[openapi(paths(...), components(...))]`。
- 可选：使用 `utoipa-gen` 的 `#[derive(ToResponse)]` 简化错误响应声明。

## 提交前检查

```bash
bash .codebuddy/skills/axum-backend/scripts/check.sh
```

等价于：`cargo fmt --all` → `cargo clippy --all-targets -- -D warnings` → `cargo test` → `cargo sqlx prepare --check`（若已安装 sqlx-cli）。
