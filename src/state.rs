//! 全局状态：通过 `State<AppState>` 注入，禁止使用全局单例

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub settings: Arc<Settings>,
}

impl AppState {
    pub fn new(pool: PgPool, settings: Settings) -> Self {
        Self {
            pool,
            settings: Arc::new(settings),
        }
    }
}
