use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{ActorId, NoteId};

pub type ExportId = Ulid;
pub type ImportId = Ulid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    Json,
    Csv,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportScope {
    All,
    Notes,
    Follows,
    Clips,
    Antennas,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub scope: ExportScope,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportStatus {
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Export {
    pub id: ExportId,
    pub user_id: ActorId,
    pub scope: ExportScope,
    pub format: ExportFormat,
    pub status: ExportStatus,
    pub file_url: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImportStatus {
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    pub file_id: Ulid,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Import {
    pub id: ImportId,
    pub user_id: ActorId,
    pub file_id: Ulid,
    pub overwrite: bool,
    pub status: ImportStatus,
    pub imported_notes: i32,
    pub imported_follows: i32,
    pub imported_clips: i32,
    pub imported_antennas: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserExportData {
    pub user_id: ActorId,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub exported_at: DateTime<Utc>,
    pub notes: Vec<ExportedNote>,
    pub follows: Vec<ExportedFollow>,
    pub clips: Vec<ExportedClip>,
    pub antennas: Vec<ExportedAntenna>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedNote {
    pub id: NoteId,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub visibility: String,
    pub cw: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedFollow {
    pub following_id: ActorId,
    pub following_username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedClip {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub note_ids: Vec<NoteId>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedAntenna {
    pub name: String,
    pub keywords: Vec<String>,
    pub exclude_keywords: Vec<String>,
    pub source: String,
    pub case_sensitive: bool,
    pub with_replies: bool,
    pub with_renotes: bool,
}
