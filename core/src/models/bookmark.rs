use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{ActorId, NoteId};

pub type BookmarkId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: BookmarkId,
    pub user_id: ActorId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkResponse {
    pub id: BookmarkId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
}

impl From<Bookmark> for BookmarkResponse {
    fn from(b: Bookmark) -> Self {
        Self {
            id: b.id,
            note_id: b.note_id,
            created_at: b.created_at,
        }
    }
}
