//! Mithic Worker
//!
//! バックグラウンドワーカープロセス。
//! フェデレーション配送キューの並列処理とリトライスケジューラを担当する。

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use mithic_federation::{ActivityDelivery, FederationService};
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

    let apalis_conn = apalis_redis::connect(config.dragonfly_url.clone()).await?;
    let storage = RedisStorage::new(apalis_conn);

    let federation_service = mithic_federation::FederationService::new(
        surreal_client,
        dragonfly_client,
        storage.clone(),
        http_client,
        config.instance_url.clone(),
    );

    info!("Worker started. Running monitor...");

    #[allow(deprecated)]
    Monitor::new()
        .register_with_count(DELIVERY_CONCURRENCY, {
            WorkerBuilder::new("federation-delivery-worker")
                .data(federation_service)
                .backend(storage)
                .build_fn(deliver_activity_job)
        })
        .run()
        .await?;

    Ok(())
}

async fn deliver_activity_job(
    job: ActivityDelivery,
    service: Data<FederationService>,
) -> Result<(), apalis::prelude::Error> {
    service.process_delivery_task(job).await
}
