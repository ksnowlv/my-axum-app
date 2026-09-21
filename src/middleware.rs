//! 中间件

use axum::{
    extract::Request,
    http::header::{AUTHORIZATION, HeaderValue},
    middleware::Next,
    response::Response,
};

use crate::error::AppError;
use crate::security;
use crate::state::AppState;

/// Bearer Token 鉴权：校验通过后把用户 ID 写入请求扩展，供 `CurrentUser` 读取
pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = bearer_token(req.headers().get(AUTHORIZATION)).ok_or(AppError::Unauthorized)?;

    let claims = security::parse_token(&state.settings.jwt_secret, token)?;

    req.extensions_mut().insert(claims.sub);
    Ok(next.run(req).await)
}

fn bearer_token(value: Option<&HeaderValue>) -> Option<&str> {
    value
        .and_then(|v| v.to_str().ok())?
        .strip_prefix("Bearer ")
        .filter(|s| !s.is_empty())
}
