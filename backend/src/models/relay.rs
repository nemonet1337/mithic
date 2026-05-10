//! Relay model
//!
//! Stores relay server information for ActivityPub federation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Relay ID
pub type RelayId = Ulid;

/// Relay status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RelayStatus {
    /// Waiting for acceptance
    Requesting,
    /// Accepted by relay
    Accepted,
    /// Rejected by relay
    Rejected,
}

impl Default for RelayStatus {
    fn default() -> Self {
        RelayStatus::Requesting
    }
}

/// Relay server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relay {
    pub id: RelayId,

    /// Inbox URL of the relay
    pub inbox: String,

    /// Current status
    #[serde(default)]
    pub status: RelayStatus,

    /// When the relay was added
    pub created_at: DateTime<Utc>,

    /// When the relay status was last updated
    pub updated_at: Option<DateTime<Utc>>,
}

impl Relay {
    /// Create a new relay
    pub fn new(inbox: String) -> Self {
        Self {
            id: RelayId::new(),
            inbox,
            status: RelayStatus::Requesting,
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    /// Mark as accepted
    pub fn accept(&mut self) {
        self.status = RelayStatus::Accepted;
        self.updated_at = Some(Utc::now());
    }

    /// Mark as rejected
    pub fn reject(&mut self) {
        self.status = RelayStatus::Rejected;
        self.updated_at = Some(Utc::now());
    }

    /// Check if relay is accepted
    pub fn is_accepted(&self) -> bool {
        self.status == RelayStatus::Accepted
    }
}

/// Create relay request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRelayRequest {
    pub inbox: String,
}

/// Relay response
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
