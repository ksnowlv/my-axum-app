//! 密码哈希与 JWT 签发/校验

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppResult;

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 ID
    pub sub: Uuid,
    /// 签发时间（秒）
    pub iat: i64,
    /// 过期时间（秒）
    pub exp: i64,
}

pub fn sign_token(secret: &str, user_id: Uuid, ttl_hours: i64) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp(),
        exp: (now + Duration::hours(ttl_hours)).timestamp(),
    };
    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn parse_token(secret: &str, token: &str) -> AppResult<Claims> {
    Ok(decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?
    .claims)
}

/// token 的绝对过期时间，便于计算剩余有效期
pub fn expiry_of(claims: &Claims) -> DateTime<Utc> {
    DateTime::from_timestamp(claims.exp, 0).unwrap_or_else(Utc::now)
}
