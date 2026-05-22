use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{actor::ActorId, note::NoteId};

pub type NoteUnreadId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteUnread {
    pub id: NoteUnreadId,
    pub note_id: NoteId,
    pub user_id: ActorId,
    pub is_specified: bool,
    pub is_mention: bool,
    pub created_at: DateTime<Utc>,
}

impl NoteUnread {
    pub fn new(note_id: NoteId, user_id: ActorId, is_specified: bool, is_mention: bool) -> Self {
        Self { id: NoteUnreadId::new(), note_id, user_id, is_specified, is_mention, created_at: Utc::now() }
    }
}
