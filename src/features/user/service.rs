//! 用户域业务层：业务规则与事务边界

use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use super::model::{AuthResponse, LoginRequest, RegisterRequest, UpdateProfileRequest, User};
use super::repository;
use crate::error::{AppError, AppResult};
use crate::security;

/// 用户注册
#[instrument(skip(pool, req), fields(email = %req.email))]
pub async fn register(
    pool: &PgPool,
    req: RegisterRequest,
    jwt_secret: &str,
    ttl_hours: i64,
) -> AppResult<AuthResponse> {
    if repository::find_by_email(pool, &req.email).await?.is_some() {
        return Err(AppError::Conflict("email already registered".into()));
    }

    let password_hash = security::hash_password(&req.password)?;
    let row = repository::create(pool, &req.email, &password_hash, &req.nickname).await?;

    issue_token(row.into(), jwt_secret, ttl_hours)
}

/// 用户登录
#[instrument(skip(pool, req), fields(email = %req.email))]
pub async fn login(
    pool: &PgPool,
    req: LoginRequest,
    jwt_secret: &str,
    ttl_hours: i64,
) -> AppResult<AuthResponse> {
    let row = repository::find_by_email(pool, &req.email)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    // 邮箱不存在与密码错误返回同一错误，避免用户枚举
    if !security::verify_password(&req.password, &row.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    if row.status != 1 {
        return Err(AppError::BadRequest("account is disabled".into()));
    }

    issue_token(row.into(), jwt_secret, ttl_hours)
}

/// 查询用户信息
#[instrument(skip(pool))]
pub async fn get_by_id(pool: &PgPool, id: Uuid) -> AppResult<User> {
    Ok(repository::find_by_id(pool, id).await?.into())
}

/// 更新当前用户资料
#[instrument(skip(pool, req))]
pub async fn update_profile(pool: &PgPool, id: Uuid, req: UpdateProfileRequest) -> AppResult<User> {
    let row =
        repository::update(pool, id, req.nickname.as_deref(), req.avatar_url.as_deref()).await?;
    Ok(row.into())
}

fn issue_token(user: User, jwt_secret: &str, ttl_hours: i64) -> AppResult<AuthResponse> {
    let token = security::sign_token(jwt_secret, user.id, ttl_hours)?;
    let claims = security::parse_token(jwt_secret, &token)?;
    Ok(AuthResponse {
        token,
        expires_at: security::expiry_of(&claims),
        user,
    })
}
