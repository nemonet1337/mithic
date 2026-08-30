use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RelayStatus {
    #[default]
    Requesting,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relay {
    pub id: String,
    pub inbox: String,
    #[serde(default)]
    pub status: RelayStatus,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl Relay {
    pub fn new(inbox: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            inbox,
            status: RelayStatus::default(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
        }
    }
}
