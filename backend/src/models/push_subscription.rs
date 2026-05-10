//! Push Subscription model
//!
//! Stores Web Push subscription data for users.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

/// Push Subscription ID
pub type PushSubscriptionId = Ulid;

/// Web Push subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscription {
    pub id: PushSubscriptionId,

    /// User ID who owns this subscription
    pub user_id: ActorId,

    /// Push service endpoint URL
    pub endpoint: String,

    /// P256DH key (base64)
    pub p256dh: String,

    /// Auth key (base64)
    pub auth: String,

    /// When the subscription was created
    pub created_at: DateTime<Utc>,

    /// When the subscription was last used
    pub last_used_at: Option<DateTime<Utc>>,
}

impl PushSubscription {
    /// Create a new push subscription
    pub fn new(user_id: ActorId, endpoint: String, p256dh: String, auth: String) -> Self {
        Self {
            id: PushSubscriptionId::new(),
            user_id,
            endpoint,
            p256dh,
            auth,
            created_at: Utc::now(),
            last_used_at: None,
        }
    }

    /// Update last used timestamp
    pub fn update_last_used(&mut self) {
        self.last_used_at = Some(Utc::now());
    }
}

/// Create push subscription request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePushSubscriptionRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

/// Push subscription response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionResponse {
    pub id: String,
    pub endpoint: String,
    pub created_at: DateTime<Utc>,
}

impl From<PushSubscription> for PushSubscriptionResponse {
    fn from(sub: PushSubscription) -> Self {
        Self {
            id: sub.id.to_string(),
            endpoint: sub.endpoint.clone(),
            created_at: sub.created_at,
        }
    }
}

/// Web Push payload
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPushPayload {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub body: WebPushBody,
}

/// Web Push notification body
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPushBody {
    pub id: String,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub tag: Option<String>,
    pub url: Option<String>,
}

impl WebPushBody {
    pub fn new(id: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            icon: None,
            badge: None,
            tag: None,
            url: None,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}
