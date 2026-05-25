use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;
use super::note::NoteId;

pub type RenoteId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Renote {
    pub id: RenoteId,
    pub created_at: DateTime<Utc>,
    pub note_id: NoteId,
    pub actor_id: ActorId,
    pub renote_note_id: NoteId,
    pub text: Option<String>,
    pub is_remote: bool,
    pub uri: Option<String>,
}

impl Renote {
    pub fn new(note_id: NoteId, actor_id: ActorId, renote_note_id: NoteId) -> Self {
        Self {
            id: RenoteId::new(),
            created_at: Utc::now(),
            note_id,
            actor_id,
            renote_note_id,
            text: None,
            is_remote: false,
            uri: None,
        }
    }

    pub fn new_with_text(
        note_id: NoteId,
        actor_id: ActorId,
        renote_note_id: NoteId,
        text: String,
    ) -> Self {
        Self {
            id: RenoteId::new(),
            created_at: Utc::now(),
            note_id,
            actor_id,
            renote_note_id,
            text: Some(text),
            is_remote: false,
            uri: None,
        }
    }
}
