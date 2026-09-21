//! 根路由组装与中间件栈

use axum::{Router, extract::Request, routing::get};
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::{docs, handlers};

const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn build() -> Router {
    let request_id_header = axum::http::HeaderName::from_static(REQUEST_ID_HEADER);

    Router::new()
        .route("/", get(handlers::hello))
        .route("/health", get(handlers::health))
        .merge(docs::router())
        .layer(
            ServiceBuilder::new()
                // 1. 为无 request id 的请求生成 UUID，并写入请求头与扩展
                .layer(SetRequestIdLayer::new(
                    request_id_header.clone(),
                    MakeRequestUuid,
                ))
                // 2. 请求 span + 响应日志（耗时、状态码）
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|req: &Request| {
                            let request_id = req
                                .headers()
                                .get(REQUEST_ID_HEADER)
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("-");
                            tracing::info_span!(
                                "request",
                                method = %req.method(),
                                uri = %req.uri(),
                                request_id = %request_id,
                            )
                        })
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Millis),
                        ),
                )
                // 3. 把 request id 回写到响应头，便于端到端串联
                .layer(PropagateRequestIdLayer::new(request_id_header)),
        )
}
