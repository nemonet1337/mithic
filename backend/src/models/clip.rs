//! Clip model
//!
//! Stores user-created clips for saving favorite notes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;
use super::note::NoteId;

/// Clip ID
pub type ClipId = Ulid;

/// Clip membership ID (note in a clip)
pub type ClipNoteId = Ulid;

/// User-created clip for saving favorite notes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: ClipId,

    /// Owner user ID
    pub user_id: ActorId,

    /// Clip name
    pub name: String,

    /// Whether the clip is public
    pub is_public: bool,

    /// Description/notes about the clip
    pub description: Option<String>,

    /// When the clip was created
    pub created_at: DateTime<Utc>,

    /// When the clip was last updated
    pub updated_at: Option<DateTime<Utc>>,
}

impl Clip {
    /// Create a new clip
    pub fn new(user_id: ActorId, name: String, is_public: bool, description: Option<String>) -> Self {
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

    /// Update the clip
    pub fn update(&mut self, name: Option<String>, is_public: Option<bool>, description: Option<Option<String>>) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(is_public) = is_public {
            self.is_public = is_public;
        }
        if let Some(description) = description {
            self.description = description;
        }
        self.updated_at = Some(Utc::now());
    }
}

/// Clip membership (note in a clip)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipNote {
    pub id: ClipNoteId,

    /// Clip ID
    pub clip_id: ClipId,

    /// Note ID
    pub note_id: NoteId,

    /// When the note was added to the clip
    pub created_at: DateTime<Utc>,
}

impl ClipNote {
    /// Create a new clip note
    pub fn new(clip_id: ClipId, note_id: NoteId) -> Self {
        Self {
            id: ClipNoteId::new(),
            clip_id,
            note_id,
            created_at: Utc::now(),
        }
    }
}

/// Create clip request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClipRequest {
    pub name: String,
    #[serde(default)]
    pub is_public: bool,
    pub description: Option<String>,
}

/// Update clip request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClipRequest {
    pub name: Option<String>,
    pub is_public: Option<bool>,
    pub description: Option<Option<String>>,
}

/// Clip response
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
    fn from(clip: Clip) -> Self {
        Self {
            id: clip.id.to_string(),
            name: clip.name,
            is_public: clip.is_public,
            description: clip.description,
            created_at: clip.created_at,
            updated_at: clip.updated_at,
        }
    }
}

/// Add note to clip request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddNoteToClipRequest {
    pub note_id: String,
}

/// Clip with notes
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipWithNotes {
    pub id: String,
    pub name: String,
    pub is_public: bool,
    pub description: Option<String>,
    pub notes: Vec<String>, // Note IDs
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Public clip response (for other users)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicClipResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
