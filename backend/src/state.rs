use axum::extract::FromRef;
use redis::aio::MultiplexedConnection as RedisConnection;
use std::sync::Arc;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use crate::{
    config::AppConfig,
    db::{DragonflyClient, SurrealClient},
    middleware::RateLimiter,
    services::{AntennaService, BookmarkService, ChartService, EmojiService, ExportService, FilterService, InstanceService, OAuthService, RelayService, WebPushService},
};

/// アプリケーション状態
#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug)]
struct AppStateInner {
    pub surreal: Surreal<Any>,
    pub dragonfly: RedisConnection,
    pub config: AppConfig,
    pub web_push: WebPushService,
    pub relay_service: RelayService,
    pub chart_service: ChartService,
    pub antenna_service: AntennaService,
    pub bookmark_service: BookmarkService,
    pub filter_service: FilterService,
    pub instance_service: InstanceService,
    pub oauth_service: OAuthService,
    pub emoji_service: EmojiService,
    pub export_service: ExportService,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(
        surreal: Surreal<Any>,
        dragonfly: RedisConnection,
        config: AppConfig,
    ) -> anyhow::Result<Self> {
        let web_push = WebPushService::new(&config)?;
        
        // Wrap clients for RelayService
        let surreal_client = crate::db::SurrealClient(surreal.clone());
        let dragonfly_client = crate::db::DragonflyClient(dragonfly.clone());
        
        let relay_service = RelayService::new(
            surreal_client.clone(),
            dragonfly_client.clone(),
            config.instance_url.clone(),
        );
        
        let chart_service = ChartService::new(
            surreal_client.clone(),
            dragonfly_client.clone(),
        );
        
        let bookmark_service = BookmarkService::new(
            surreal_client.clone(),
            dragonfly_client.clone(),
        );
        
        let filter_service = FilterService::new(
            surreal_client.clone(),
            dragonfly_client.clone(),
        );
        
        let instance_service = InstanceService::new(
            surreal_client.clone(),
            dragonfly_client.clone(),
        );
        
        let oauth_service = OAuthService::new(
            surreal_client.clone(),
            dragonfly_client.clone(),
        );
        
        let antenna_service = AntennaService::new(
            surreal_client.clone(),
            dragonfly_client.clone(),
        );
        
        let emoji_service = EmojiService::new(
            surreal_client.clone(),
            dragonfly_client.clone(),
        );
        
        let export_service = ExportService::new(
            surreal_client,
            dragonfly,
        );
        
        let rate_limiter = RateLimiter::new();
        
        Ok(Self {
            inner: Arc::new(AppStateInner {
                surreal,
                dragonfly,
                config,
                web_push,
                relay_service,
                chart_service,
                antenna_service,
                bookmark_service,
                filter_service,
                instance_service,
                oauth_service,
                emoji_service,
                export_service,
                rate_limiter,
            }),
        })
    }

    pub fn surreal(&self) -> &Surreal<Any> {
        &self.inner.surreal
    }

    pub fn dragonfly(&self) -> &RedisConnection {
        &self.inner.dragonfly
    }

    pub fn config(&self) -> &AppConfig {
        &self.inner.config
    }
    
    pub fn web_push_service(&self) -> &WebPushService {
        &self.inner.web_push
    }
    
    pub fn relay_service(&self) -> &RelayService {
        &self.inner.relay_service
    }
    
    pub fn chart_service(&self) -> &ChartService {
        &self.inner.chart_service
    }
    
    pub fn antenna_service(&self) -> &AntennaService {
        &self.inner.antenna_service
    }
    
    pub fn emoji_service(&self) -> &EmojiService {
        &self.inner.emoji_service
    }
    
    pub fn bookmark_service(&self) -> &BookmarkService {
        &self.inner.bookmark_service
    }
    
    pub fn filter_service(&self) -> &FilterService {
        &self.inner.filter_service
    }
    
    pub fn instance_service(&self) -> &InstanceService {
        &self.inner.instance_service
    }
    
    pub fn oauth_service(&self) -> &OAuthService {
        &self.inner.oauth_service
    }
    
    pub fn export_service(&self) -> &ExportService {
        &self.inner.export_service
    }

    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.inner.rate_limiter
    }
}

/// 認証済みユーザー情報
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: ulid::Ulid,
    pub username: String,
    pub is_admin: bool,
}
