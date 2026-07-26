use anyhow::Result;
use apalis_redis::RedisStorage;
use tracing::info;

// mimalloc をグローバルアロケータに設定 (TODO Phase 0)
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = mithic_config::AppConfig::from_env()?;

    info!(
        "Connecting to SurrealDB at {} (pool size {})",
        config.surrealdb_endpoint, config.surrealdb_pool_size
    );
    let surreal_config = mithic_db::SurrealConfig::from(&config);
    let surreal_client =
        mithic_db::create_pool(&surreal_config, config.surrealdb_pool_size).await?;

    info!("Initializing SurrealDB schema");
    mithic_db::init_schema(surreal_client.get()).await?;

    info!("Connecting to Dragonfly at {}", config.dragonfly_url);
    let dragonfly_client = mithic_db::create_dragonfly_client(&config.dragonfly_url).await?;
    let apalis_conn = apalis_redis::connect(config.dragonfly_url.clone()).await?;
    let storage = RedisStorage::new(apalis_conn);

    let state =
        mithic_api::AppState::new(surreal_client, dragonfly_client, storage, config.clone())?;
    let app = mithic_api::routes::create_router(state);

    let addr = format!("0.0.0.0:{}", config.server_port);
    info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    // ConnectInfo を有効化 (レート制限のクライアント IP 用)
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Shutdown signal received");
}
