use serde::{Deserialize, Serialize};

use super::User;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum NoteVisibility {
    #[default]
    Public,
    Home,
    Followers,
    Specified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionSummary {
    pub emoji: String,
    pub count: u64,
    pub reacted_by_me: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAttachment {
    pub id: String,
    pub url: String,
    pub preview_url: Option<String>,
    pub media_type: String,
    pub alt: Option<String>,
    pub is_sensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub created_at: String,
    pub author: User,
    pub content: String,
    pub cw: Option<String>,
    pub visibility: NoteVisibility,
    pub reactions: Vec<ReactionSummary>,
    pub reply_count: u64,
    pub renote_count: u64,
    pub quote_count: u64,
    pub attachments: Vec<MediaAttachment>,
    pub tags: Vec<String>,
    pub is_nsfw: bool,
    /// リノート先ノート ID（pure renote / quote 共通）
    #[serde(default)]
    pub renote_id: Option<String>,
    /// ネストされた元ノート（表示用）
    #[serde(default)]
    pub renote: Option<Box<Note>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteRequest {
    pub text: String,
    pub visibility: NoteVisibility,
    pub cw: Option<String>,
    pub is_nsfw: bool,
    pub file_ids: Vec<String>,
    pub reply_id: Option<String>,
    pub poll_choices: Vec<String>,
    pub scheduled_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionRequest {
    pub note_id: String,
    pub reaction: String,
}
