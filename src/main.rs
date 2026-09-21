use my_axum_app::{config::Settings, logging, router, state::AppState};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load()?;
    logging::init(&settings.log_level, settings.log_json);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&settings.database_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;

    let addr = settings.addr.clone();
    let app = router::build(AppState::new(pool, settings));

    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        anyhow::anyhow!(
            "无法绑定 {addr}: {e}；端口可能被其它进程占用，\
             可用 `lsof -nP -iTCP:{port} -sTCP:LISTEN` 排查",
            port = addr.rsplit(':').next().unwrap_or("3000")
        )
    })?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    tracing::info!(
        "api docs: http://{}/swagger-ui, openapi: http://{}/api-docs/openapi.json",
        listener.local_addr()?,
        listener.local_addr()?
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
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
