use std::sync::Arc;

use mithic_config::AppConfig;
use mithic_db::{DragonflyClient, SurrealClient};
use mithic_federation::FederationService;

use crate::middleware::RateLimiter;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug)]
struct AppStateInner {
    pub surreal: SurrealClient,
    pub dragonfly: DragonflyClient,
    pub config: AppConfig,
    pub http_client: reqwest::Client,
    pub federation_service: FederationService,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(
        surreal: SurrealClient,
        dragonfly: DragonflyClient,
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
            config.instance_url.clone(),
        );

        let rate_limiter = RateLimiter::new();

        Ok(Self {
            inner: Arc::new(AppStateInner {
                surreal,
                dragonfly,
                config,
                http_client,
                federation_service,
                rate_limiter,
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
}

pub use mithic_core::AuthUser as ApiAuthUser;
