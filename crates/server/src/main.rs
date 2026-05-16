use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = mithic_config::AppConfig::from_env()?;

    info!("Connecting to SurrealDB at {}", config.surrealdb_endpoint);
    let surreal = mithic_db::create_surreal_client(mithic_db::SurrealConfig {
        endpoint: config.surrealdb_endpoint.clone(),
        namespace: config.surrealdb_namespace.clone(),
        database: config.surrealdb_database.clone(),
        username: config.surrealdb_username.clone(),
        password: config.surrealdb_password.clone(),
    })
    .await?;

    info!("Connecting to Dragonfly at {}", config.dragonfly_url);
    let dragonfly = mithic_db::create_dragonfly_client(&config.dragonfly_url).await?;

    let surreal_client = mithic_db::SurrealClient(surreal);
    let dragonfly_client = mithic_db::DragonflyClient(dragonfly);

    let state = mithic_api::AppState::new(surreal_client, dragonfly_client, config.clone())?;
    let app = mithic_api::routes::create_router(state);

    let addr = format!("0.0.0.0:{}", config.server_port);
    info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
