//! 统一响应包装

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

/// 统一成功响应：`{ code, message, data }`
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    /// 业务码，0 表示成功
    pub code: u16,
    pub message: String,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".to_owned(),
            data,
        }
    }

    pub fn with_message(message: impl Into<String>, data: T) -> Self {
        Self {
            code: 0,
            message: message.into(),
            data,
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
