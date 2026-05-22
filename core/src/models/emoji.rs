use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type EmojiId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EmojiCategory {
    General, Activities, Animals, Flags, Food, Objects,
    People, Nature, Symbols, Uncategorized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    pub id: EmojiId,
    pub name: String,
    pub category: EmojiCategory,
    pub aliases: Vec<String>,
    pub url: String,
    pub file_id: Ulid,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEmojiRequest {
    pub name: String,
    pub category: EmojiCategory,
    pub aliases: Vec<String>,
    pub file_id: Ulid,
    pub is_public: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmojiRequest {
    pub name: Option<String>,
    pub category: Option<EmojiCategory>,
    pub aliases: Option<Vec<String>>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmojiResponse {
    pub id: EmojiId,
    pub name: String,
    pub category: EmojiCategory,
    pub aliases: Vec<String>,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Emoji> for EmojiResponse {
    fn from(e: Emoji) -> Self {
        Self { id: e.id, name: e.name, category: e.category, aliases: e.aliases, url: e.url, width: e.width, height: e.height, is_public: e.is_public, created_at: e.created_at, updated_at: e.updated_at }
    }
}
