//! __feature__ 域：路由组装

pub mod handler;
pub mod model;
pub mod repository;
pub mod service;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::state::AppState;

/// 返回 `Router<AppState>`，由 `src/router.rs` 统一 `nest` 到 `/api/v1/__features__`
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/__features__", get(handler::list).post(handler::create))
        .route(
            "/__features__/{id}",
            get(handler::detail)
                .patch(handler::update)
                .delete(handler::remove),
        )
}
