use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type RelayId = Ulid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RelayStatus {
    Requesting,
    Accepted,
    Rejected,
}

impl Default for RelayStatus {
    fn default() -> Self {
        RelayStatus::Requesting
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relay {
    pub id: RelayId,
    pub inbox: String,
    #[serde(default)]
    pub status: RelayStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Relay {
    pub fn new(inbox: String) -> Self {
        Self {
            id: RelayId::new(),
            inbox,
            status: RelayStatus::Requesting,
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    pub fn accept(&mut self) {
        self.status = RelayStatus::Accepted;
        self.updated_at = Some(Utc::now());
    }
    pub fn reject(&mut self) {
        self.status = RelayStatus::Rejected;
        self.updated_at = Some(Utc::now());
    }
    pub fn is_accepted(&self) -> bool {
        self.status == RelayStatus::Accepted
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRelayRequest {
    pub inbox: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayResponse {
    pub id: String,
    pub inbox: String,
    pub status: RelayStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Relay> for RelayResponse {
    fn from(relay: Relay) -> Self {
        Self {
            id: relay.id.to_string(),
            inbox: relay.inbox.clone(),
            status: relay.status,
            created_at: relay.created_at,
            updated_at: relay.updated_at,
        }
    }
}
