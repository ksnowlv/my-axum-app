//! 用户域数据模型：数据库实体 + DTO

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 数据库实体（含敏感字段，不对外序列化）
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 对外的用户信息
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            nickname: row.nickname,
            avatar_url: row.avatar_url,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// 注册请求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
    #[validate(length(min = 8, max = 64, message = "密码长度需为 8-64 位"))]
    pub password: String,
    #[validate(length(min = 1, max = 32, message = "昵称长度需为 1-32 位"))]
    pub nickname: String,
}

/// 登录请求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
    #[validate(length(min = 8, max = 64))]
    pub password: String,
}

/// 更新资料请求：未提供的字段保持不变
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 32, message = "昵称长度需为 1-32 位"))]
    pub nickname: Option<String>,
    #[validate(url(message = "头像地址需为合法 URL"))]
    pub avatar_url: Option<String>,
}

/// 登录响应
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    /// token 过期时间（RFC3339）
    pub expires_at: DateTime<Utc>,
    pub user: User,
}
