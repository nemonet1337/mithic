//! Mithic Worker
//!
//! バックグラウンドワーカープロセス。
//! フェデレーション配送キューの並列処理とリトライスケジューラを担当する。

use tracing::info;

// mimalloc をグローバルアロケータに設定 (TODO Phase 0)
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// 配送ワーカーの並列数
const DELIVERY_CONCURRENCY: usize = 4;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Mithic Worker...");

    let config = mithic_config::AppConfig::from_env()?;

    info!("Connecting to SurrealDB at {}", config.surrealdb_endpoint);
    let surreal_config = mithic_db::SurrealConfig {
        endpoint: config.surrealdb_endpoint.clone(),
        namespace: config.surrealdb_namespace.clone(),
        database: config.surrealdb_database.clone(),
        username: config.surrealdb_username.clone(),
        password: config.surrealdb_password.clone(),
    };
    let surreal_client = mithic_db::create_pool(&surreal_config, 2).await?;

    info!("Initializing SurrealDB schema");
    mithic_db::init_schema(surreal_client.get()).await?;

    info!("Connecting to Dragonfly at {}", config.dragonfly_url);
    let dragonfly_client = mithic_db::create_dragonfly_client(&config.dragonfly_url).await?;

    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(32)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .build()
        .unwrap_or_default();

    let federation_service = mithic_federation::FederationService::new(
        surreal_client,
        dragonfly_client,
        http_client,
        config.instance_url.clone(),
    );

    // 配送ワーカー (並列) + リトライスケジューラを起動
    tokio::spawn(async move {
        if let Err(e) = federation_service
            .run_delivery_workers(DELIVERY_CONCURRENCY)
            .await
        {
            tracing::error!("Federation delivery workers failed: {}", e);
        }
    });

    info!("Worker started. Waiting for jobs...");

    tokio::signal::ctrl_c().await?;
    info!("Worker shutting down...");

    Ok(())
}
