mod logging;

use axum::{Router, extract::Request, routing::get};
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Level;

const REQUEST_ID_HEADER: &str = "x-request-id";

#[tokio::main]
async fn main() {
    // 必须在构建任何异步任务之前初始化，否则后续日志会丢失
    logging::init();

    let request_id_header = axum::http::HeaderName::from_static(REQUEST_ID_HEADER);

    let app = Router::new()
        .route("/", get(hello))
        .route("/health", get(health))
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
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind 127.0.0.1:3000");

    tracing::info!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

#[tracing::instrument]
async fn hello() -> &'static str {
    tracing::info!("handling hello request");
    "Hello, World!"
}

async fn health() -> &'static str {
    "ok"
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received, starting graceful shutdown");
}
