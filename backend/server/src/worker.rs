//! 連合配送ワーカー（同一プロセス内で HTTP と並走）

use std::time::Duration;

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use futures::future;
use mithic_federation::{ActivityDelivery, FederationService, DLQ_KEY};
use tower::retry::Policy;
use tracing::{info, warn};

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

/// 配送キューを消費する（ブロックする）。HTTP サーバーと並走させる想定。
pub async fn run_delivery_worker(
    storage: RedisStorage<ActivityDelivery>,
    federation_service: FederationService,
) -> anyhow::Result<()> {
    info!(
        "Delivery worker started (concurrency={}, max_retries={}, dlq={})",
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
