use object_store::ObjectStore;
use std::sync::Arc;

use apalis_redis::RedisStorage;
use mithic_config::AppConfig;
use mithic_db::{DragonflyClient, SurrealClient};
use mithic_federation::{ActivityDelivery, FederationService};

use crate::events::{StreamBroadcast, StreamReceiver, StreamSender};
use crate::middleware::RateLimiter;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub surreal: SurrealClient,
    pub dragonfly: DragonflyClient,
    pub config: AppConfig,
    pub http_client: reqwest::Client,
    pub federation_service: FederationService,
    pub rate_limiter: RateLimiter,
    pub stream_tx: StreamSender,
    pub storage: Arc<dyn ObjectStore>,
}

impl std::fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("surreal", &self.surreal)
            .field("dragonfly", &self.dragonfly)
            .field("config", &self.config)
            .field("http_client", &self.http_client)
            .field("federation_service", &self.federation_service)
            .field("rate_limiter", &self.rate_limiter)
            .field("stream_tx", &self.stream_tx)
            .finish_non_exhaustive()
    }
}

impl AppState {
    pub fn new(
        surreal: SurrealClient,
        dragonfly: DragonflyClient,
        storage: RedisStorage<ActivityDelivery>,
        config: AppConfig,
    ) -> anyhow::Result<Self> {
        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(32)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_default();

        let federation_service = FederationService::new(
            surreal.clone(),
            dragonfly.clone(),
            storage,
            http_client.clone(),
            config.instance_url.clone(),
        );

        let rate_limiter = RateLimiter::new();
        let stream_tx = crate::events::channel();
        let storage = mithic_db::create_storage_client(&config)?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                surreal,
                dragonfly,
                config,
                http_client,
                federation_service,
                rate_limiter,
                stream_tx,
                storage,
            }),
        })
    }

    pub fn surreal(&self) -> &SurrealClient {
        &self.inner.surreal
    }
    pub fn dragonfly(&self) -> &DragonflyClient {
        &self.inner.dragonfly
    }
    pub fn config(&self) -> &AppConfig {
        &self.inner.config
    }
    pub fn http_client(&self) -> &reqwest::Client {
        &self.inner.http_client
    }
    pub fn federation_service(&self) -> &FederationService {
        &self.inner.federation_service
    }
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.inner.rate_limiter
    }
    pub fn storage(&self) -> &Arc<dyn ObjectStore> {
        &self.inner.storage
    }

    /// ストリームイベントを購読する
    pub fn subscribe_stream(&self) -> StreamReceiver {
        self.inner.stream_tx.subscribe()
    }

    /// ストリームイベントを発行する (購読者ゼロは無視)
    pub fn publish_stream(&self, event: StreamBroadcast) {
        let _ = self.inner.stream_tx.send(event);
    }
}

pub use mithic_core::AuthUser as ApiAuthUser;
