//! Web Push Notification Service
//!
//! Handles Web Push notifications using the web-push crate.

use std::sync::Arc;
use tracing::{error, info, warn};
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder,
    WebPushClient, WebPushError, WebPushMessageBuilder,
};

use crate::{
    config::AppConfig,
    error::{AppError, Result},
    models::{PushSubscription, WebPushBody, WebPushPayload},
};

/// Web Push service
#[derive(Clone)]
pub struct WebPushService {
    client: Arc<IsahcWebPushClient>,
    vapid_private_key: Option<String>,
    vapid_public_key: Option<String>,
    subject: String,
}

impl WebPushService {
    /// Create a new Web Push service
    pub fn new(config: &AppConfig) -> Result<Self> {
        let client = IsahcWebPushClient::new()
            .map_err(|e| AppError::Internal(format!("Failed to create WebPush client: {}", e)))?;

        // VAPID keys are optional - if not provided, push notifications won't work
        let vapid_private_key = std::env::var("VAPID_PRIVATE_KEY").ok();
        let vapid_public_key = std::env::var("VAPID_PUBLIC_KEY").ok();

        if vapid_private_key.is_none() {
            warn!("VAPID_PRIVATE_KEY not set - push notifications disabled");
        }

        Ok(Self {
            client: Arc::new(client),
            vapid_private_key,
            vapid_public_key,
            subject: format!("mailto:{}", config.admin_email.clone().unwrap_or_else(|| "admin@example.com".to_string())),
        })
    }

    /// Get VAPID public key for client-side subscription
    pub fn get_public_key(&self) -> Option<&String> {
        self.vapid_public_key.as_ref()
    }

    /// Check if push notifications are enabled
    pub fn is_enabled(&self) -> bool {
        self.vapid_private_key.is_some()
    }

    /// Send push notification to a subscription
    pub async fn send_notification(
        &self,
        subscription: &PushSubscription,
        payload: &WebPushPayload,
    ) -> Result<()> {
        if !self.is_enabled() {
            return Err(AppError::Internal("Push notifications not configured".to_string()));
        }

        // Build subscription info
        let subscription_info = SubscriptionInfo::new(
            &subscription.endpoint,
            &subscription.p256dh,
            &subscription.auth,
        );

        // Build VAPID signature
        let private_key = self.vapid_private_key.as_ref().unwrap();
        let sig_builder = VapidSignatureBuilder::from_base64(private_key, &subscription_info)
            .map_err(|e| AppError::Internal(format!("Failed to build VAPID signature: {}", e)))?;

        let signature = sig_builder
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to sign: {}", e)))?;

        // Build message
        let payload_json = serde_json::to_string(payload)
            .map_err(|e| AppError::Internal(format!("Failed to serialize payload: {}", e)))?;

        let mut message_builder = WebPushMessageBuilder::new(&subscription_info);
        message_builder.set_payload(ContentEncoding::Aes128Gcm, payload_json.as_bytes());
        message_builder.set_vapid_signature(signature);
        message_builder.set_ttl(86400); // 24 hours

        let message = message_builder
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to build message: {}", e)))?;

        // Send
        match self.client.send(message).await {
            Ok(_) => {
                info!("Push notification sent successfully to {}", subscription.endpoint);
                Ok(())
            }
            Err(WebPushError::EndpointNotValid(_) | WebPushError::EndpointNotFound(_)) => {
                warn!("Invalid endpoint - subscription should be removed: {}", subscription.endpoint);
                Err(AppError::Internal("Invalid endpoint".to_string()))
            }
            Err(e) => {
                error!("Failed to send push notification: {}", e);
                Err(AppError::Internal(format!("Push notification failed: {}", e)))
            }
        }
    }

    /// Send notification to all user subscriptions
    pub async fn send_to_user(
        &self,
        db: &crate::db::SurrealDb,
        user_id: &crate::models::ActorId,
        notification_type: impl Into<String>,
        body: WebPushBody,
    ) -> Result<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Get user's subscriptions
        let subscriptions: Vec<PushSubscription> = db
            .query("SELECT * FROM push_subscription WHERE user_id = $user_id")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        if subscriptions.is_empty() {
            return Ok(());
        }

        let payload = WebPushPayload {
            msg_type: notification_type.into(),
            body,
        };

        // Send to all subscriptions
        for subscription in &subscriptions {
            match self.send_notification(subscription, &payload).await {
                Ok(_) => {}
                Err(AppError::Internal(msg)) if msg.contains("Invalid endpoint") => {
                    // Remove invalid subscription
                    let _ = db
                        .query("DELETE push_subscription WHERE id = $id")
                        .bind(("id", subscription.id.to_string()))
                        .await;
                }
                Err(e) => {
                    error!("Failed to send to subscription {}: {}", subscription.id, e);
                }
            }
        }

        Ok(())
    }

    /// Send mention notification
    pub async fn send_mention(
        &self,
        db: &crate::db::SurrealDb,
        user_id: &crate::models::ActorId,
        actor_name: &str,
        note_url: &str,
    ) -> Result<()> {
        let body = WebPushBody::new(
            ulid::Ulid::new().to_string(),
            format!("{} mentioned you", actor_name),
            "You were mentioned in a note".to_string(),
        )
        .with_url(note_url.to_string());

        self.send_to_user(db, user_id, "mention", body).await
    }

    /// Send follow notification
    pub async fn send_follow(
        &self,
        db: &crate::db::SurrealDb,
        user_id: &crate::models::ActorId,
        actor_name: &str,
        actor_url: &str,
    ) -> Result<()> {
        let body = WebPushBody::new(
            ulid::Ulid::new().to_string(),
            format!("{} followed you", actor_name),
            "You have a new follower".to_string(),
        )
        .with_url(actor_url.to_string());

        self.send_to_user(db, user_id, "follow", body).await
    }

    /// Send reaction notification
    pub async fn send_reaction(
        &self,
        db: &crate::db::SurrealDb,
        user_id: &crate::models::ActorId,
        actor_name: &str,
        emoji: &str,
        note_url: &str,
    ) -> Result<()> {
        let body = WebPushBody::new(
            ulid::Ulid::new().to_string(),
            format!("{} reacted {}", actor_name, emoji),
            "Someone reacted to your note".to_string(),
        )
        .with_url(note_url.to_string());

        self.send_to_user(db, user_id, "reaction", body).await
    }
}
