use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;
use super::note::NoteId;

pub type ClipId = Ulid;
pub type ClipNoteId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: ClipId,
    pub user_id: ActorId,
    pub name: String,
    pub is_public: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Clip {
    pub fn new(
        user_id: ActorId,
        name: String,
        is_public: bool,
        description: Option<String>,
    ) -> Self {
        Self {
            id: ClipId::new(),
            user_id,
            name,
            is_public,
            description,
            created_at: Utc::now(),
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipNote {
    pub id: ClipNoteId,
    pub clip_id: ClipId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
}

impl ClipNote {
    pub fn new(clip_id: ClipId, note_id: NoteId) -> Self {
        Self {
            id: ClipNoteId::new(),
            clip_id,
            note_id,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClipRequest {
    pub name: String,
    #[serde(default)]
    pub is_public: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClipRequest {
    pub name: Option<String>,
    pub is_public: Option<bool>,
    pub description: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipResponse {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Clip> for ClipResponse {
    fn from(c: Clip) -> Self {
        Self {
            id: c.id.to_string(),
            name: c.name,
            is_public: c.is_public,
            description: c.description,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddNoteToClipRequest {
    pub note_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipWithNotes {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub description: Option<String>,
    pub notes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicClipResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
