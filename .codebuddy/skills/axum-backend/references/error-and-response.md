# 统一错误与统一响应

## response.rs：ApiResponse\<T\>

```rust
use serde::Serialize;

/// 统一成功响应包装：`{ code, message, data }`
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { code: 0, message: "ok".into(), data }
    }
    pub fn with_message(message: impl Into<String>, data: T) -> Self {
        Self { code: 0, message: message.into(), data }
    }
}

/// 空数据场景（删除、登出）
pub type EmptyResponse = ApiResponse<()>;
pub fn empty() -> EmptyResponse { ApiResponse::ok(()) }

impl<T: Serialize> axum::response::IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}
```

约定：

- `code` 为业务码，`0` 表示成功；HTTP 状态码仍由 `StatusCode` 表达。
- 分页统一 `Page<T> { items: Vec<T>, total: i64, page: i64, page_size: i64 }`。
- 允许原始响应的例外：`/health`、静态文件、Swagger UI。

## error.rs：AppError

```rust
use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error")]
    Internal(#[source] anyhow::Error),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn code(&self) -> u16 { self.status().as_u16() }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 5xx 打印完整错误链；4xx 仅 debug 级别
        match &self {
            Self::Internal(e) => tracing::error!(error = %e, "internal error"),
            other => tracing::debug!(error = %other, "request rejected"),
        }
        let body = ErrorBody {
            code: self.code(),
            message: self.to_string(),
            details: None,
        };
        (self.status(), Json(body)).into_response()
    }
}
```

## 错误转换

```rust
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                AppError::Conflict("resource already exists".into())
            }
            sqlx::Error::Database(db) if db.is_foreign_key_violation() => {
                AppError::BadRequest("related resource missing".into())
            }
            other => AppError::Internal(other.into()),
        }
    }
}

impl From<anyhow::Error> for AppError { fn from(e: anyhow::Error) -> Self { Self::Internal(e) } }
impl From<validator::ValidationErrors> for AppError {
    fn from(e: validator::ValidationErrors) -> Self { Self::Validation(e.to_string()) }
}
```

要点：

- 数据库错误**必须**在此转换，禁止让 `sqlx::Error` 泄漏到 handler。
- 唯一约束冲突映射为 409，外键冲突映射为 400。
- 内部错误对外只暴露 "internal error"，详细信息仅写入日志。
- 校验错误详情可放进 `details` 字段（如字段级错误 map），但不得回显密码等敏感值。

## 业务错误的使用方式

```rust
// service 层
if repository::exists_by_email(pool, &email).await? {
    return Err(AppError::Conflict("email already registered".into()));
}
```

- 需要新增错误语义时先扩展 `AppError` 枚举，禁止在 handler 里直接拼 `(StatusCode::XXX, Json(...))`。
- 所有对外错误消息使用英文（日志同理），面向用户的中文文案由前端维护。
