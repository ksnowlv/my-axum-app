use my_axum_app::router;

#[tokio::main]
async fn main() {
    // 必须在构建任何异步任务之前初始化，否则后续日志会丢失
    my_axum_app::logging::init();

    let app = router::build();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind 127.0.0.1:3000");

    tracing::info!("listening on http://{}", listener.local_addr().unwrap());
    tracing::info!(
        "api docs: http://{}/swagger-ui, openapi: http://{}/api-docs/openapi.json",
        listener.local_addr().unwrap(),
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
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
