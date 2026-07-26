//! Mithic Worker
//!
//! バックグラウンドワーカープロセス。
//! フェデレーション配送キューの並列処理とリトライスケジューラを担当する。

use std::time::Duration;

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use futures::future;
use mithic_federation::{ActivityDelivery, FederationService, DLQ_KEY};
use tower::retry::Policy;
use tracing::{info, warn};

// mimalloc をグローバルアロケータに設定
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// 配送ワーカーの並列数
const DELIVERY_CONCURRENCY: usize = 4;
/// 最大リトライ回数
const MAX_DELIVERY_RETRIES: usize = 5;

/// 指数バックオフ付きリトライ (1s, 2s, 4s, 8s, … 最大 60s)
#[derive(Clone, Debug)]
struct BackoffRetryPolicy {
    max_retries: usize,
}

impl BackoffRetryPolicy {
    fn new(max_retries: usize) -> Self {
        Self { max_retries }
    }
}

impl Default for BackoffRetryPolicy {
    fn default() -> Self {
        Self::new(MAX_DELIVERY_RETRIES)
    }
}

impl<T, Res, Ctx> Policy<Request<T, Ctx>, Res, Error> for BackoffRetryPolicy
where
    T: Clone + Send + 'static,
    Ctx: Clone + Send + 'static,
{
    type Future = future::BoxFuture<'static, ()>;

    fn retry(
        &mut self,
        req: &mut Request<T, Ctx>,
        result: &mut Result<Res, Error>,
    ) -> Option<Self::Future> {
        match result {
            Ok(_) => None,
            Err(_) if self.max_retries == 0 => None,
            Err(_) if self.max_retries.saturating_sub(req.parts.attempt.current()) > 0 => {
                let attempt = req.parts.attempt.current() as u32;
                let secs = 1u64.saturating_mul(2u64.saturating_pow(attempt)).min(60);
                Some(Box::pin(async move {
                    tokio::time::sleep(Duration::from_secs(secs)).await;
                }))
            }
            Err(_) => None,
        }
    }

    fn clone_request(&mut self, req: &Request<T, Ctx>) -> Option<Request<T, Ctx>> {
        let req = req.clone();
        req.parts.attempt.increment();
        Some(req)
    }
}

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
    let surreal_config = mithic_db::SurrealConfig::from(&config);
    let surreal_client = mithic_db::create_pool(&surreal_config, 2).await?;

    info!("Initializing SurrealDB schema");
    mithic_db::init_schema(surreal_client.get()).await?;

    info!("Connecting to Dragonfly at {}", config.dragonfly_url);
    let dragonfly_client = mithic_db::create_dragonfly_client(&config.dragonfly_url).await?;

    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(32)
        .pool_idle_timeout(Duration::from_secs(90))
        .build()?;

    let apalis_conn = apalis_redis::connect(config.dragonfly_url.clone()).await?;
    let storage = RedisStorage::new(apalis_conn);

    let federation_service = mithic_federation::FederationService::new(
        surreal_client,
        dragonfly_client,
        storage.clone(),
        http_client,
        config.instance_url.clone(),
    );

    info!(
        "Worker started (concurrency={}, max_retries={}, dlq={})",
        DELIVERY_CONCURRENCY, MAX_DELIVERY_RETRIES, DLQ_KEY
    );

    Monitor::new()
        .register({
            WorkerBuilder::new("federation-delivery-worker")
                .concurrency(DELIVERY_CONCURRENCY)
                .retry(BackoffRetryPolicy::default())
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
    match service.process_delivery_task(job).await {
        Ok(()) => Ok(()),
        Err(e) => {
            warn!("Delivery job failed (retry if attempts remain): {e}");
            Err(e)
        }
    }
}
