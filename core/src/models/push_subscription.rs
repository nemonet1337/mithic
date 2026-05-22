use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

pub type PushSubscriptionId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscription {
    pub id: PushSubscriptionId,
    pub user_id: ActorId,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl PushSubscription {
    pub fn new(user_id: ActorId, endpoint: String, p256dh: String, auth: String) -> Self {
        Self { id: PushSubscriptionId::new(), user_id, endpoint, p256dh, auth, created_at: Utc::now(), last_used_at: None }
    }

    pub fn update_last_used(&mut self) { self.last_used_at = Some(Utc::now()); }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePushSubscriptionRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionResponse {
    pub id: String,
    pub endpoint: String,
    pub created_at: DateTime<Utc>,
}

impl From<PushSubscription> for PushSubscriptionResponse {
    fn from(sub: PushSubscription) -> Self {
        Self { id: sub.id.to_string(), endpoint: sub.endpoint.clone(), created_at: sub.created_at }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPushPayload {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub body: WebPushBody,
}

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
        Self { id: id.into(), title: title.into(), body: body.into(), icon: None, badge: None, tag: None, url: None }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self { self.icon = Some(icon.into()); self }
    pub fn with_url(mut self, url: impl Into<String>) -> Self { self.url = Some(url.into()); self }
}
