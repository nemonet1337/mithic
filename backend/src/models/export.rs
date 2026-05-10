//! Import/Export models
//!
//! Data export and import functionality for user data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{ActorId, NoteId};

/// Export format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

/// Export scope
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportScope {
    /// All data
    All,
    /// Notes only
    Notes,
    /// Follows only
    Follows,
    /// Clips only
    Clips,
    /// Antennas only
    Antennas,
}

/// Create export request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExportRequest {
    pub scope: ExportScope,
    pub format: ExportFormat,
}

/// Export status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportStatus {
    /// Processing
    Processing,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

/// Export record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Export {
    pub id: Ulid,
    pub user_id: ActorId,
    pub scope: ExportScope,
    pub format: ExportFormat,
    pub status: ExportStatus,
    pub file_url: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Export response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResponse {
    pub id: Ulid,
    pub scope: ExportScope,
    pub format: ExportFormat,
    pub status: ExportStatus,
    pub file_url: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<Export> for ExportResponse {
    fn from(export: Export) -> Self {
        Self {
            id: export.id,
            scope: export.scope,
            format: export.format,
            status: export.status,
            file_url: export.file_url,
            error_message: export.error_message,
            created_at: export.created_at,
            completed_at: export.completed_at,
        }
    }
}

/// User export data
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

/// Exported note
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedNote {
    pub id: NoteId,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub visibility: String,
    pub cw: Option<String>,
}

/// Exported follow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedFollow {
    pub following_id: ActorId,
    pub following_username: String,
    pub created_at: DateTime<Utc>,
}

/// Exported clip
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedClip {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub note_ids: Vec<NoteId>,
    pub created_at: DateTime<Utc>,
}

/// Exported antenna
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

/// Import request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    pub file_id: Ulid,
    pub overwrite: bool,
}

/// Import status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImportStatus {
    /// Processing
    Processing,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

/// Import record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Import {
    pub id: Ulid,
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

/// Import response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResponse {
    pub id: Ulid,
    pub status: ImportStatus,
    pub imported_notes: i32,
    pub imported_follows: i32,
    pub imported_clips: i32,
    pub imported_antennas: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<Import> for ImportResponse {
    fn from(import: Import) -> Self {
        Self {
            id: import.id,
            status: import.status,
            imported_notes: import.imported_notes,
            imported_follows: import.imported_follows,
            imported_clips: import.imported_clips,
            imported_antennas: import.imported_antennas,
            error_message: import.error_message,
            created_at: import.created_at,
            completed_at: import.completed_at,
        }
    }
}
