//! OpenAPI 文档定义与 Swagger UI 路由
//!
//! 新增接口时：
//! 1. handler 上添加 `#[utoipa::path(...)]`
//! 2. 把 handler 注册到下方 `paths(...)`
//! 3. 把请求/响应中的结构体注册到 `components(schemas(...))`

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::Hello;
use crate::response::ApiResponse;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "my-axum-app API",
        version = "0.1.0",
        description = "Axum 后端服务接口文档，由 utoipa 从代码注解自动生成"
    ),
    paths(crate::handlers::hello, crate::handlers::health),
    components(schemas(ApiResponse<Hello>)),
    tags(
        (name = "demo", description = "示例接口"),
        (name = "system", description = "系统接口")
    )
)]
pub struct ApiDoc;

/// 挂载 `/swagger-ui` 与 `/api-docs/openapi.json`
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
