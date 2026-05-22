use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;
use validator::Validate;

use super::actor::ActorId;

pub type NoteId = Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoteVisibility {
    Public,
    Home,
    Followers,
    Specified,
}

impl Default for NoteVisibility {
    fn default() -> Self {
        NoteVisibility::Public
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: NoteId,
    pub created_at: DateTime<Utc>,
    pub reply_id: Option<NoteId>,
    pub renote_id: Option<NoteId>,
    #[validate(length(max = 8192))]
    pub text: Option<String>,
    #[validate(length(max = 512))]
    pub cw: Option<String>,
    pub actor_id: ActorId,
    pub renote_count: i32,
    pub replies_count: i32,
    pub reactions: HashMap<String, i32>,
    pub visibility: NoteVisibility,
    #[validate(length(max = 512))]
    pub uri: Option<String>,
    pub file_ids: Vec<String>,
    pub visible_user_ids: Vec<ActorId>,
    pub mentions: Vec<ActorId>,
    pub emojis: Vec<String>,
    pub tags: Vec<String>,
    pub has_poll: bool,
    pub actor_host: Option<String>,
    pub reply_actor_id: Option<ActorId>,
    pub renote_actor_id: Option<ActorId>,
}

impl Note {
    pub fn new(actor_id: ActorId, text: Option<String>, visibility: NoteVisibility) -> Self {
        Self {
            id: NoteId::new(),
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
            actor_host: None,
            reply_actor_id: None,
            renote_actor_id: None,
        }
    }

    pub fn is_renote(&self) -> bool {
        self.renote_id.is_some()
    }
    pub fn is_reply(&self) -> bool {
        self.reply_id.is_some()
    }
    pub fn has_text(&self) -> bool {
        self.text.as_ref().map(|t| !t.is_empty()).unwrap_or(false)
    }
    pub fn has_files(&self) -> bool {
        !self.file_ids.is_empty()
    }
    pub fn total_reactions(&self) -> i32 {
        self.reactions.values().sum()
    }
}
