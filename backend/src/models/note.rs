use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

use super::actor::ActorId;

pub type NoteId = Ulid;
pub use shared::NoteVisibility;

/// TEXT カラム → `NoteVisibility`。DB には小文字 1 語で保存する。
pub fn visibility_str(visibility: NoteVisibility) -> &'static str {
    match visibility {
        NoteVisibility::Public => "public",
        NoteVisibility::Home => "home",
        NoteVisibility::Followers => "followers",
        NoteVisibility::Specified => "specified",
    }
}

/// TEXT カラム → `NoteVisibility` (`shared` 型のため `FromStr` は直接 implant できない)
pub fn visibility_from_str(s: &str) -> anyhow::Result<NoteVisibility> {
    match s {
        "public" => Ok(NoteVisibility::Public),
        "home" => Ok(NoteVisibility::Home),
        "followers" => Ok(NoteVisibility::Followers),
        "specified" => Ok(NoteVisibility::Specified),
        other => Err(anyhow::anyhow!("unknown note visibility: {other}")),
    }
}

// DB の snake_case フィールドと一致させるため rename しない
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub reply_id: Option<NoteId>,
    #[serde(default)]
    pub renote_id: Option<NoteId>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub cw: Option<String>,
    pub actor_id: ActorId,
    #[serde(default)]
    pub renote_count: i32,
    #[serde(default)]
    pub replies_count: i32,
    #[serde(default)]
    pub reactions: HashMap<String, i32>,
    #[serde(default)]
    pub visibility: NoteVisibility,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub file_ids: Vec<String>,
    #[serde(default)]
    pub visible_user_ids: Vec<ActorId>,
    #[serde(default)]
    pub mentions: Vec<ActorId>,
    #[serde(default)]
    pub emojis: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub has_poll: bool,
    #[serde(default)]
    pub host: Option<String>,
}

impl Note {
    pub fn new(actor_id: ActorId, text: Option<String>, visibility: NoteVisibility) -> Self {
        Self {
            id: NoteId::generate(),
            created_at: Utc::now(),
            reply_id: None,
            renote_id: None,
            text,
            cw: None,
            actor_id,
            renote_count: 0,
            replies_count: 0,
            reactions: HashMap::new(),
            visibility,
            uri: None,
            file_ids: Vec::new(),
            visible_user_ids: Vec::new(),
            mentions: Vec::new(),
            emojis: Vec::new(),
            tags: Vec::new(),
            has_poll: false,
            host: None,
        }
    }
}
