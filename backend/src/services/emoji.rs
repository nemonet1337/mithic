//! Emoji service for custom emoji management
//!
//! Provides upload, management, and retrieval of instance custom emojis.

use tracing::{error, info};

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{
        CreateEmojiRequest, Emoji, EmojiCategory, EmojiId, UpdateEmojiRequest,
    },
};

/// Emoji service for managing custom emojis
#[derive(Debug, Clone)]
pub struct EmojiService {
    surreal: SurrealClient,
    dragonfly: DragonflyClient,
}

impl EmojiService {
    /// Create a new emoji service
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal,
            dragonfly,
        }
    }

    /// Create a new custom emoji
    pub async fn create_emoji(&self, request: CreateEmojiRequest) -> Result<Emoji> {
        // Get the file to get URL and dimensions
        let file: Option<crate::models::DriveFile> = self
            .surreal
            .select(("drive_file", request.file_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let file = file.ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

        let url = file.url.ok_or_else(|| AppError::BadRequest("File has no URL".to_string()))?;

        let emoji = Emoji {
            id: EmojiId::new(),
            name: request.name,
            category: request.category,
            aliases: request.aliases,
            url,
            file_id: request.file_id,
            width: file.width,
            height: file.height,
            is_public: request.is_public,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };

        self.surreal
            .create::<Option<Emoji>>(("emoji", emoji.id.to_string()))
            .content(emoji.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created emoji {} from file {}", emoji.name, file.id);

        Ok(emoji)
    }

    /// Update an emoji
    pub async fn update_emoji(&self, emoji_id: &EmojiId, request: UpdateEmojiRequest) -> Result<Emoji> {
        let mut emoji: Emoji = self
            .surreal
            .select(("emoji", emoji_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("Emoji not found".to_string()))?;

        if let Some(name) = request.name {
            emoji.name = name;
        }
        if let Some(category) = request.category {
            emoji.category = category;
        }
        if let Some(aliases) = request.aliases {
            emoji.aliases = aliases;
        }
        if let Some(is_public) = request.is_public {
            emoji.is_public = is_public;
        }
        emoji.updated_at = Some(chrono::Utc::now());

        self.surreal
            .update::<Option<Emoji>>(("emoji", emoji_id.to_string()))
            .content(emoji.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Updated emoji {}", emoji_id);

        Ok(emoji)
    }

    /// Delete an emoji
    pub async fn delete_emoji(&self, emoji_id: &EmojiId) -> Result<()> {
        self.surreal
            .delete(("emoji", emoji_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Deleted emoji {}", emoji_id);

        Ok(())
    }

    /// Get all public emojis
    pub async fn get_public_emojis(&self) -> Result<Vec<Emoji>> {
        let emojis: Vec<Emoji> = self
            .surreal
            .query("SELECT * FROM emoji WHERE is_public = true ORDER BY name ASC")
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(emojis)
    }

    /// Get all emojis (including private)
    pub async fn get_all_emojis(&self) -> Result<Vec<Emoji>> {
        let emojis: Vec<Emoji> = self
            .surreal
            .query("SELECT * FROM emoji ORDER BY name ASC")
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(emojis)
    }

    /// Get a specific emoji
    pub async fn get_emoji(&self, emoji_id: &EmojiId) -> Result<Emoji> {
        let emoji: Option<Emoji> = self
            .surreal
            .select(("emoji", emoji_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        emoji.ok_or_else(|| AppError::NotFound("Emoji not found".to_string()))
    }

    /// Get emojis by category
    pub async fn get_emojis_by_category(&self, category: EmojiCategory) -> Result<Vec<Emoji>> {
        let emojis: Vec<Emoji> = self
            .surreal
            .query("SELECT * FROM emoji WHERE category = $category AND is_public = true ORDER BY name ASC")
            .bind(("category", format!("{:?}", category).to_lowercase()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(emojis)
    }

    /// Search emojis by name or alias
    pub async fn search_emojis(&self, query: &str) -> Result<Vec<Emoji>> {
        let emojis: Vec<Emoji> = self
            .surreal
            .query("SELECT * FROM emoji WHERE is_public = true AND (name ~ $query OR $query IN aliases) ORDER BY name ASC")
            .bind(("query", query))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(emojis)
    }

    /// Copy a remote emoji to local
    pub async fn copy_emoji(&self, emoji_id: &EmojiId) -> Result<Emoji> {
        let source_emoji: Emoji = self
            .surreal
            .select(("emoji", emoji_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("Emoji not found".to_string()))?;

        // Create a new local emoji
        let new_emoji = Emoji {
            id: EmojiId::new(),
            name: source_emoji.name.clone(),
            category: source_emoji.category.clone(),
            aliases: vec![], // Reset aliases for local copy
            url: source_emoji.url.clone(),
            file_id: source_emoji.file_id.clone(),
            width: source_emoji.width,
            height: source_emoji.height,
            is_public: true,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };

        self.surreal
            .create::<Option<Emoji>>(("emoji", new_emoji.id.to_string()))
            .content(new_emoji.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Copied emoji {} from remote to local", source_emoji.name);

        Ok(new_emoji)
    }

    /// List remote emojis
    pub async fn list_remote_emojis(&self, host: Option<String>, limit: usize) -> Result<Vec<Emoji>> {
        let query = if let Some(h) = host {
            "SELECT * FROM emoji WHERE host = $host ORDER BY name ASC LIMIT $limit"
        } else {
            "SELECT * FROM emoji WHERE host IS NOT NULL ORDER BY name ASC LIMIT $limit"
        };

        let mut surreal_query = self.surreal.query(query).bind(("limit", limit));
        if let Some(h) = host {
            surreal_query = surreal_query.bind(("host", h));
        }

        let emojis: Vec<Emoji> = surreal_query
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(emojis)
    }
}
