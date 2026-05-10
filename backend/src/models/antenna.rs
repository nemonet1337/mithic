//! Antenna models for keyword monitoring
//!
//! Allows users to set up keyword-based monitoring of notes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{ActorId, NoteId};

/// Antenna ID
pub type AntennaId = Ulid;

/// Antenna source type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AntennaSource {
    /// All notes
    All,
    /// Home timeline only
    Home,
    /// Users list
    Users,
    /// Specific users
    UsersList,
}

/// Antenna ID
pub type AntennaNoteId = Ulid;

/// Antenna model - keyword monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Antenna {
    pub id: AntennaId,
    pub user_id: ActorId,
    pub name: String,
    pub source: AntennaSource,
    pub keywords: Vec<String>,
    pub exclude_keywords: Vec<String>,
    pub users: Vec<ActorId>,
    pub user_list_id: Option<Ulid>,
    pub case_sensitive: bool,
    pub with_replies: bool,
    pub with_renotes: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Create antenna request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAntennaRequest {
    pub name: String,
    pub source: AntennaSource,
    pub keywords: Vec<String>,
    pub exclude_keywords: Vec<String>,
    pub users: Vec<ActorId>,
    pub user_list_id: Option<Ulid>,
    pub case_sensitive: bool,
    pub with_replies: bool,
    pub with_renotes: bool,
}

/// Update antenna request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAntennaRequest {
    pub name: Option<String>,
    pub source: Option<AntennaSource>,
    pub keywords: Option<Vec<String>>,
    pub exclude_keywords: Option<Vec<String>>,
    pub users: Option<Vec<ActorId>>,
    pub user_list_id: Option<Ulid>,
    pub case_sensitive: Option<bool>,
    pub with_replies: Option<bool>,
    pub with_renotes: Option<bool>,
}

/// Antenna response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntennaResponse {
    pub id: AntennaId,
    pub name: String,
    pub source: AntennaSource,
    pub keywords: Vec<String>,
    pub exclude_keywords: Vec<String>,
    pub users: Vec<ActorId>,
    pub user_list_id: Option<Ulid>,
    pub case_sensitive: bool,
    pub with_replies: bool,
    pub with_renotes: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Antenna> for AntennaResponse {
    fn from(antenna: Antenna) -> Self {
        Self {
            id: antenna.id,
            name: antenna.name,
            source: antenna.source,
            keywords: antenna.keywords,
            exclude_keywords: antenna.exclude_keywords,
            users: antenna.users,
            user_list_id: antenna.user_list_id,
            case_sensitive: antenna.case_sensitive,
            with_replies: antenna.with_replies,
            with_renotes: antenna.with_renotes,
            created_at: antenna.created_at,
            updated_at: antenna.updated_at,
        }
    }
}

/// Antenna note - captured note matching antenna criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntennaNote {
    pub id: AntennaNoteId,
    pub antenna_id: AntennaId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
}

/// Antenna note response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntennaNoteResponse {
    pub id: AntennaNoteId,
    pub antenna_id: AntennaId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
}

impl From<AntennaNote> for AntennaNoteResponse {
    fn from(note: AntennaNote) -> Self {
        Self {
            id: note.id,
            antenna_id: note.antenna_id,
            note_id: note.note_id,
            created_at: note.created_at,
        }
    }
}
