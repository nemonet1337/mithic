use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type ActivityId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: ActivityId,
    pub uri: String,
    pub activity_type: String,
    pub actor_id: Option<String>,
    pub note_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Activity {
    pub fn new(
        uri: String,
        activity_type: String,
        actor_id: Option<String>,
        note_id: Option<String>,
    ) -> Self {
        Self {
            id: ActivityId::new(),
            uri,
            activity_type,
            actor_id,
            note_id,
            created_at: Utc::now(),
        }
    }
}
