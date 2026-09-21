//! OpenAPI 文档定义与 Swagger UI 路由
//!
//! 新增接口时：
//! 1. handler 上添加 `#[utoipa::path(...)]`
//! 2. 把 handler 注册到下方 `paths(...)`
//! 3. 把请求/响应中的结构体注册到 `components(schemas(...))`

use axum::Router;
use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::error::ErrorBody;
use crate::features::user::model::{
    AuthResponse, LoginRequest, RegisterRequest, UpdateProfileRequest, User,
};
use crate::handlers::Hello;
use crate::response::ApiResponse;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "my-axum-app API",
        version = "0.1.0",
        description = "Axum 后端服务接口文档，由 utoipa 从代码注解自动生成"
    ),
    paths(
        crate::handlers::hello,
        crate::handlers::health,
        crate::features::user::handler::register,
        crate::features::user::handler::login,
        crate::features::user::handler::detail,
        crate::features::user::handler::me,
        crate::features::user::handler::update_me,
    ),
    modifiers(&SecurityAddon),
    components(schemas(
        ApiResponse<Hello>,
        ApiResponse<User>,
        ApiResponse<AuthResponse>,
        ErrorBody,
        User,
        AuthResponse,
        RegisterRequest,
        LoginRequest,
        UpdateProfileRequest,
    )),
    tags(
        (name = "demo", description = "示例接口"),
        (name = "system", description = "系统接口"),
        (name = "user", description = "用户接口")
    )
)]
pub struct ApiDoc;

/// utoipa 5 的 `components(...)` 只接受 schemas/responses，
/// 安全方案需通过 `Modify` 注入
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

/// 挂载 `/swagger-ui` 与 `/api-docs/openapi.json`
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
