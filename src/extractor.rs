//! 通用 Extractor

use axum::{
    Json,
    extract::{FromRequest, FromRequestParts},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::{AppError, AppResult};

/// 当前登录用户 ID，由鉴权中间件写入请求扩展
pub struct CurrentUser(pub uuid::Uuid);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> AppResult<Self> {
        parts
            .extensions
            .get::<uuid::Uuid>()
            .copied()
            .map(CurrentUser)
            .ok_or(AppError::Unauthorized)
    }
}

/// 反序列化 JSON 并自动执行 `validator` 校验
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}
