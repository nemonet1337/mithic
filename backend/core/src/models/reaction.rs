use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{actor::ActorId, note::NoteId};

pub type ReactionId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    pub id: ReactionId,
    pub created_at: DateTime<Utc>,
    pub note_id: NoteId,
    pub actor_id: ActorId,
    pub reaction: String,
    pub is_remote: bool,
    pub uri: Option<String>,
}

impl Reaction {
    pub fn new(note_id: NoteId, actor_id: ActorId, reaction: String) -> Self {
        Self {
            id: ReactionId::new(),
            created_at: Utc::now(),
            note_id,
            actor_id,
            reaction,
            is_remote: false,
            uri: None,
        }
    }

    pub fn new_remote(note_id: NoteId, actor_id: ActorId, reaction: String, uri: String) -> Self {
        Self {
            id: ReactionId::new(),
            created_at: Utc::now(),
            note_id,
            actor_id,
            reaction,
            is_remote: true,
            uri: Some(uri),
        }
    }
}
