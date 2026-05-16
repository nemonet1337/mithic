use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

pub type MuteId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mute {
    pub id: MuteId,
    pub muter_id: ActorId,
    pub mutee_id: ActorId,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Mute {
    pub fn new(muter_id: ActorId, mutee_id: ActorId, expires_at: Option<DateTime<Utc>>) -> Self {
        Self { id: MuteId::new(), muter_id, mutee_id, created_at: Utc::now(), expires_at }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    pub fn is_active(&self) -> bool { !self.is_expired() }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMuteRequest {
    pub user_id: String,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MuteResponse {
    pub id: String,
    pub muter_id: String,
    pub mutee_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<Mute> for MuteResponse {
    fn from(mute: Mute) -> Self {
        Self {
            id: mute.id.to_string(),
            muter_id: mute.muter_id.to_string(),
            mutee_id: mute.mutee_id.to_string(),
            created_at: mute.created_at,
            expires_at: mute.expires_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuteListQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl MuteListQuery {
    pub fn limit(&self) -> i32 { self.limit.unwrap_or(20).min(100) }
    pub fn offset(&self) -> i32 { self.offset.unwrap_or(0) }
}
