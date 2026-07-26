use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type UsedUsernameId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsedUsername {
    pub id: UsedUsernameId,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl UsedUsername {
    pub fn new(username: String) -> Self {
        Self {
            id: UsedUsernameId::new(),
            username: username.to_lowercase(),
            created_at: Utc::now(),
        }
    }
}
