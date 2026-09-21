//! 统一错误类型

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

pub type AppResult<T> = Result<T, AppError>;

/// 统一错误响应体
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("email or password is incorrect")]
    InvalidCredentials,
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("internal error")]
    Internal(#[source] anyhow::Error),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized | Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Config(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            Self::Internal(e) => tracing::error!(error = %e, "internal error"),
            Self::Config(e) => tracing::error!(error = %e, "configuration error"),
            other => tracing::debug!(error = %other, "request rejected"),
        }

        let status = self.status();
        (
            status,
            Json(ErrorBody {
                code: status.as_u16(),
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                Self::Conflict("resource already exists".into())
            }
            sqlx::Error::Database(db) if db.is_foreign_key_violation() => {
                Self::BadRequest("related resource missing".into())
            }
            other => Self::Internal(other.into()),
        }
    }
}

impl From<axum::extract::rejection::JsonRejection> for AppError {
    fn from(e: axum::extract::rejection::JsonRejection) -> Self {
        Self::BadRequest(e.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(e: validator::ValidationErrors) -> Self {
        Self::Validation(e.to_string())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::Internal(anyhow::anyhow!("password error: {e}"))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        tracing::debug!(error = %e, "jwt verification failed");
        Self::Unauthorized
    }
}
