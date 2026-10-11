use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type RelayId = Ulid;
pub use shared::RelayStatus;

/// TEXT カラム → `RelayStatus`。DB には小文字 1 語で保存する。
/// (`shared` 型のため `FromStr` は直接 implant できない)
pub fn relay_status_from_str(s: &str) -> anyhow::Result<RelayStatus> {
    match s {
        "requesting" => Ok(RelayStatus::Requesting),
        "accepted" => Ok(RelayStatus::Accepted),
        "rejected" => Ok(RelayStatus::Rejected),
        other => Err(anyhow::anyhow!("unknown relay status: {other}")),
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
