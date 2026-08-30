use object_store::ObjectStore;
use std::sync::Arc;

use apalis_redis::RedisStorage;
use base64::Engine;
use mithic_config::AppConfig;
use mithic_db::{DragonflyClient, SurrealClient};
use mithic_federation::{ActivityDelivery, FederationService};
use tracing::info;
use web_push::{IsahcWebPushClient, VapidSignatureBuilder};

use crate::events::{StreamBroadcast, StreamReceiver, StreamSender};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub surreal: SurrealClient,
    pub dragonfly: DragonflyClient,
    pub config: AppConfig,
    pub http_client: reqwest::Client,
    pub federation_service: FederationService,
    pub stream_tx: StreamSender,
    pub storage: Arc<dyn ObjectStore>,
    /// URL-safe base64 public key for browser PushManager.subscribe
    pub vapid_public_key: Option<String>,
    pub web_push_client: Option<IsahcWebPushClient>,
}

impl std::fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("surreal", &self.surreal)
            .field("dragonfly", &self.dragonfly)
            .field("config", &self.config)
            .field("vapid_public_key", &self.vapid_public_key)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

fn derive_vapid_public_key(private_b64: &str) -> Option<String> {
    let builder = VapidSignatureBuilder::from_base64_no_sub(private_b64).ok()?;
    let bytes = builder.get_public_key();
    Some(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

impl AppState {
    pub fn new(
        surreal: SurrealClient,
        dragonfly: DragonflyClient,
        queue_storage: RedisStorage<ActivityDelivery>,
        config: AppConfig,
    ) -> anyhow::Result<Self> {
        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(32)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()?;

        let federation_service = FederationService::new(
            surreal.clone(),
            dragonfly.clone(),
            queue_storage,
            http_client.clone(),
            config.instance_url.clone(),
        );
        federation_service.spawn_cache_janitor();

        let (vapid_public_key, web_push_client) = if let Some(ref pk) = config.vapid_private_key {
            match derive_vapid_public_key(pk) {
                Some(pub_key) => {
                    let client = IsahcWebPushClient::new()
                        .map_err(|e| anyhow::anyhow!("Web push client: {e}"))?;
                    info!("Web Push enabled (VAPID public key derived)");
                    (Some(pub_key), Some(client))
                }
                None => {
                    tracing::warn!("VAPID_PRIVATE_KEY set but invalid; Web Push disabled");
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        let stream_tx = crate::events::channel();
        let object_storage = mithic_db::create_storage_client(&config)?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                surreal,
                dragonfly,
                config,
                http_client,
                federation_service,
                stream_tx,
                storage: object_storage,
                vapid_public_key,
                web_push_client,
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
    pub fn storage(&self) -> &Arc<dyn ObjectStore> {
        &self.inner.storage
    }
    pub fn vapid_public_key(&self) -> Option<&str> {
        self.inner.vapid_public_key.as_deref()
    }
    pub fn web_push_client(&self) -> Option<&IsahcWebPushClient> {
        self.inner.web_push_client.as_ref()
    }

    pub fn subscribe_stream(&self) -> StreamReceiver {
        self.inner.stream_tx.subscribe()
    }

    pub fn publish_stream(&self, event: StreamBroadcast) {
        let _ = self.inner.stream_tx.send(event);
    }
}
