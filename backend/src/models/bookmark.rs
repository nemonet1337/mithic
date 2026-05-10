//! Bookmark models
//!
//! User bookmark functionality for saving notes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{ActorId, NoteId};

/// Bookmark ID
pub type BookmarkId = Ulid;

/// Bookmark model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: BookmarkId,
    pub user_id: ActorId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
}

/// Bookmark response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkResponse {
    pub id: BookmarkId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
}

impl From<Bookmark> for BookmarkResponse {
    fn from(bookmark: Bookmark) -> Self {
        Self {
            id: bookmark.id,
            note_id: bookmark.note_id,
            created_at: bookmark.created_at,
        }
    }
}
