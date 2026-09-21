//! 用户域：路由组装

pub mod handler;
pub mod model;
pub mod repository;
pub mod service;

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::middleware::auth_middleware;
use crate::state::AppState;

/// `public` 为匿名可访问接口，`protected` 走 Bearer Token 鉴权
pub fn router(state: AppState) -> Router<AppState> {
    let protected = Router::<AppState>::new()
        .route("/me", get(handler::me).patch(handler::update_me))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware));

    Router::new()
        .route("/register", post(handler::register))
        .route("/login", post(handler::login))
        .route("/{id}", get(handler::detail))
        .merge(protected)
}
