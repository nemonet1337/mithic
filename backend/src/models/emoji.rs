//! Custom emoji models
//!
//! Instance-wide custom emoji management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Emoji ID
pub type EmojiId = Ulid;

/// Emoji category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EmojiCategory {
    /// General purpose
    General,
    /// Activities
    Activities,
    /// Animals
    Animals,
    /// Flags
    Flags,
    /// Food
    Food,
    /// Objects
    Objects,
    /// People
    People,
    /// Nature
    Nature,
    /// Symbols
    Symbols,
    /// Uncategorized
    Uncategorized,
}

/// Custom emoji model
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

/// Create emoji request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEmojiRequest {
    pub name: String,
    pub category: EmojiCategory,
    pub aliases: Vec<String>,
    pub file_id: Ulid,
    pub is_public: bool,
}

/// Update emoji request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmojiRequest {
    pub name: Option<String>,
    pub category: Option<EmojiCategory>,
    pub aliases: Option<Vec<String>>,
    pub is_public: Option<bool>,
}

/// Emoji response
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
    fn from(emoji: Emoji) -> Self {
        Self {
            id: emoji.id,
            name: emoji.name,
            category: emoji.category,
            aliases: emoji.aliases,
            url: emoji.url,
            width: emoji.width,
            height: emoji.height,
            is_public: emoji.is_public,
            created_at: emoji.created_at,
            updated_at: emoji.updated_at,
        }
    }
}
