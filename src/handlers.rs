//! 全局处理器（业务接口后续迁移到 features/<domain>/handler.rs）

use serde::Serialize;
use utoipa::ToSchema;

use crate::response::ApiResponse;

#[derive(Debug, Serialize, ToSchema)]
pub struct Hello {
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/",
    tag = "demo",
    summary = "示例接口",
    responses(
        (status = 200, description = "返回问候语", body = ApiResponse<Hello>),
    )
)]
#[tracing::instrument]
pub async fn hello() -> ApiResponse<Hello> {
    tracing::info!("handling hello request");
    ApiResponse::ok(Hello {
        message: "Hello, World!".to_owned(),
    })
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    summary = "健康检查",
    responses(
        (status = 200, description = "服务正常", body = String),
    )
)]
pub async fn health() -> &'static str {
    "ok"
}
