use object_store::ObjectStore;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::db::{DragonflyClient, SurrealClient};
use crate::federation::{ActivityDelivery, FederationService};
use apalis_redis::RedisStorage;
use base64::Engine;
use tracing::info;
use web_push_native::jwt_simple::algorithms::{ECDSAP256KeyPairLike, ES256KeyPair};

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
    pub vapid_key_pair: Option<ES256KeyPair>,
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

fn decode_url_b64(s: &str) -> Option<Vec<u8>> {
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    engine
        .decode(s)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(s))
        .ok()
}

fn parse_vapid_key(private_b64: &str) -> Option<(ES256KeyPair, String)> {
    let raw = decode_url_b64(private_b64)?;
    let key_pair = ES256KeyPair::from_bytes(&raw).ok()?;
    let uncompressed = key_pair.key_pair().public_key().to_bytes_uncompressed();
    let pub_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(uncompressed);
    Some((key_pair, pub_b64))
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

        let (vapid_public_key, vapid_key_pair) = if let Some(ref pk) = config.vapid_private_key {
            match parse_vapid_key(pk) {
                Some((key_pair, pub_key)) => {
                    info!("Web Push enabled (VAPID public key derived)");
                    (Some(pub_key), Some(key_pair))
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
        let object_storage = crate::db::create_storage_client(&config)?;

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
                vapid_key_pair,
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
    pub fn vapid_key_pair(&self) -> Option<&ES256KeyPair> {
        self.inner.vapid_key_pair.as_ref()
    }

    pub fn subscribe_stream(&self) -> StreamReceiver {
        self.inner.stream_tx.subscribe()
    }

    pub fn publish_stream(&self, event: StreamBroadcast) {
        let _ = self.inner.stream_tx.send(event);
    }
}
