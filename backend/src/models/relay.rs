use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type RelayId = Ulid;
pub use shared::RelayStatus;

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
            id: RelayId::generate(),
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
