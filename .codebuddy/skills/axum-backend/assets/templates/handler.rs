//! __feature__ 域的 HTTP 层：参数提取 → 调用 service → 包装 ApiResponse

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::AppError;
use crate::extractor::ValidatedJson;
use crate::response::{ApiResponse, Page};
use crate::state::AppState;

use super::model::{
    Create__Feature__Request, List__Feature__Query, Update__Feature__Request, __Feature__,
};
use super::service;

/// 列表
#[utoipa::path(
    get,
    path = "/api/v1/__features__",
    tag = "__feature__",
    params(
        ("page" = Option<i64>, Query, description = "页码，从 1 开始"),
        ("page_size" = Option<i64>, Query, description = "每页条数，最大 100"),
    ),
    responses(
        (status = 200, description = "分页列表", body = ApiResponse<Page<__Feature__>>),
        (status = 400, description = "参数校验失败"),
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<List__Feature__Query>,
) -> Result<ApiResponse<Page<__Feature__>>, AppError> {
    query.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let page = service::list__Feature__s(&state.pool, query).await?;
    Ok(ApiResponse::ok(page))
}

/// 详情
#[utoipa::path(
    get,
    path = "/api/v1/__features__/{id}",
    tag = "__feature__",
    params(("id" = Uuid, Path, description = "资源 ID")),
    responses(
        (status = 200, description = "资源详情", body = ApiResponse<__Feature__>),
        (status = 404, description = "资源不存在"),
    )
)]
pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<__Feature__>, AppError> {
    let item = service::get__Feature__(&state.pool, id).await?;
    Ok(ApiResponse::ok(item))
}

/// 创建
#[utoipa::path(
    post,
    path = "/api/v1/__features__",
    tag = "__feature__",
    request_body = Create__Feature__Request,
    responses(
        (status = 201, description = "创建成功", body = ApiResponse<__Feature__>),
        (status = 400, description = "参数校验失败"),
        (status = 409, description = "资源已存在"),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<Create__Feature__Request>,
) -> Result<(StatusCode, ApiResponse<__Feature__>), AppError> {
    let item = service::create__Feature__(&state.pool, payload).await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(item)))
}

/// 更新
#[utoipa::path(
    patch,
    path = "/api/v1/__features__/{id}",
    tag = "__feature__",
    params(("id" = Uuid, Path, description = "资源 ID")),
    request_body = Update__Feature__Request,
    responses(
        (status = 200, description = "更新成功", body = ApiResponse<__Feature__>),
        (status = 404, description = "资源不存在"),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<Update__Feature__Request>,
) -> Result<ApiResponse<__Feature__>, AppError> {
    let item = service::update__Feature__(&state.pool, id, payload).await?;
    Ok(ApiResponse::ok(item))
}

/// 删除
#[utoipa::path(
    delete,
    path = "/api/v1/__features__/{id}",
    tag = "__feature__",
    params(("id" = Uuid, Path, description = "资源 ID")),
    responses(
        (status = 200, description = "删除成功"),
        (status = 404, description = "资源不存在"),
    )
)]
pub async fn remove(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<()>, AppError> {
    service::delete__Feature__(&state.pool, id).await?;
    Ok(ApiResponse::ok(()))
}
