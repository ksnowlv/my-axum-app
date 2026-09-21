//! 用户域 HTTP 层：参数提取 → 调用 service → 包装 ApiResponse

use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::error::{AppError, ErrorBody};
use crate::extractor::{CurrentUser, ValidatedJson};
use crate::response::ApiResponse;
use crate::state::AppState;

use super::model::{AuthResponse, LoginRequest, RegisterRequest, UpdateProfileRequest, User};
use super::service;

/// 用户注册
#[utoipa::path(
    post,
    path = "/api/v1/users/register",
    tag = "user",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "注册成功，返回 token", body = ApiResponse<AuthResponse>),
        (status = 400, description = "参数校验失败", body = ErrorBody),
        (status = 409, description = "邮箱已注册", body = ErrorBody),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<(StatusCode, ApiResponse<AuthResponse>), AppError> {
    let result = service::register(
        &state.pool,
        payload,
        &state.settings.jwt_secret,
        state.settings.jwt_expiration_hours,
    )
    .await?;
    Ok((StatusCode::CREATED, ApiResponse::ok(result)))
}

/// 用户登录
#[utoipa::path(
    post,
    path = "/api/v1/users/login",
    tag = "user",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登录成功，返回 token", body = ApiResponse<AuthResponse>),
        (status = 400, description = "参数校验失败", body = ErrorBody),
        (status = 401, description = "邮箱或密码错误", body = ErrorBody),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let result = service::login(
        &state.pool,
        payload,
        &state.settings.jwt_secret,
        state.settings.jwt_expiration_hours,
    )
    .await?;
    Ok(ApiResponse::ok(result))
}

/// 查询指定用户信息
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "user",
    params(("id" = Uuid, Path, description = "用户 ID")),
    responses(
        (status = 200, description = "用户信息", body = ApiResponse<User>),
        (status = 404, description = "用户不存在", body = ErrorBody),
    )
)]
pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<User>, AppError> {
    let user = service::get_by_id(&state.pool, id).await?;
    Ok(ApiResponse::ok(user))
}

/// 查询当前登录用户信息
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "user",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "当前用户信息", body = ApiResponse<User>),
        (status = 401, description = "未登录或 token 无效", body = ErrorBody),
    )
)]
pub async fn me(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<ApiResponse<User>, AppError> {
    let user = service::get_by_id(&state.pool, user_id).await?;
    Ok(ApiResponse::ok(user))
}

/// 更新当前登录用户资料
#[utoipa::path(
    patch,
    path = "/api/v1/users/me",
    tag = "user",
    security(("bearer_auth" = [])),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "更新成功", body = ApiResponse<User>),
        (status = 400, description = "参数校验失败", body = ErrorBody),
        (status = 401, description = "未登录或 token 无效", body = ErrorBody),
    )
)]
pub async fn update_me(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    ValidatedJson(payload): ValidatedJson<UpdateProfileRequest>,
) -> Result<ApiResponse<User>, AppError> {
    let user = service::update_profile(&state.pool, user_id, payload).await?;
    Ok(ApiResponse::ok(user))
}
