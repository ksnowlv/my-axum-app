//! 配置加载：`.env` → 环境变量 → 默认值

use crate::error::AppError;

const DEFAULT_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_JWT_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Clone)]
pub struct Settings {
    pub addr: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub log_level: String,
    pub log_json: bool,
}

impl Settings {
    pub fn load() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("DATABASE_URL 未设置".into()))?;
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| AppError::Config("JWT_SECRET 未设置".into()))?;

        if jwt_secret.len() < 16 {
            return Err(AppError::Config("JWT_SECRET 长度至少 16 位".into()));
        }

        Ok(Self {
            addr: std::env::var("APP_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_owned()),
            database_url,
            jwt_secret,
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_JWT_EXPIRATION_HOURS),
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            log_json: env_flag("LOG_JSON"),
        })
    }
}

fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
