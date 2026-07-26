use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{ActorId, NoteId};

pub type AntennaId = Ulid;
pub type AntennaNoteId = Ulid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AntennaSource {
    All,
    Home,
    Users,
    UsersList,
}

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
    fn from(a: Antenna) -> Self {
        Self {
            id: a.id,
            name: a.name,
            source: a.source,
            keywords: a.keywords,
            exclude_keywords: a.exclude_keywords,
            users: a.users,
            user_list_id: a.user_list_id,
            case_sensitive: a.case_sensitive,
            with_replies: a.with_replies,
            with_renotes: a.with_renotes,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntennaNote {
    pub id: AntennaNoteId,
    pub antenna_id: AntennaId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntennaNoteResponse {
    pub id: AntennaNoteId,
    pub antenna_id: AntennaId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
}

impl From<AntennaNote> for AntennaNoteResponse {
    fn from(n: AntennaNote) -> Self {
        Self {
            id: n.id,
            antenna_id: n.antenna_id,
            note_id: n.note_id,
            created_at: n.created_at,
        }
    }
}
