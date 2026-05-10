//! Bookmark service for managing user bookmarks
//!
//! Provides functionality for saving and managing note bookmarks.

use tracing::info;

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{ActorId, Bookmark, BookmarkId, NoteId},
};

/// Bookmark service for managing bookmarks
#[derive(Debug, Clone)]
pub struct BookmarkService {
    surreal: SurrealClient,
    dragonfly: DragonflyClient,
}

impl BookmarkService {
    /// Create a new bookmark service
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal,
            dragonfly,
        }
    }

    /// Create a bookmark
    pub async fn create_bookmark(
        &self,
        user_id: &ActorId,
        note_id: &NoteId,
    ) -> Result<Bookmark> {
        // Check if already bookmarked
        let existing: Option<Bookmark> = self
            .surreal
            .query("SELECT * FROM bookmark WHERE user_id = $user_id AND note_id = $note_id LIMIT 1")
            .bind(("user_id", user_id.to_string()))
            .bind(("note_id", note_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        if existing.is_some() {
            return Err(AppError::BadRequest("Note already bookmarked".to_string()));
        }

        let bookmark = Bookmark {
            id: ulid::Ulid::new(),
            user_id: user_id.clone(),
            note_id: note_id.clone(),
            created_at: chrono::Utc::now(),
        };

        self.surreal
            .create::<Option<Bookmark>>(("bookmark", bookmark.id.to_string()))
            .content(bookmark.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created bookmark {} for user {}", bookmark.id, user_id);

        Ok(bookmark)
    }

    /// Delete a bookmark by note ID
    pub async fn delete_bookmark(
        &self,
        user_id: &ActorId,
        note_id: &NoteId,
    ) -> Result<()> {
        let bookmarks: Vec<Bookmark> = self
            .surreal
            .query("SELECT * FROM bookmark WHERE user_id = $user_id AND note_id = $note_id")
            .bind(("user_id", user_id.to_string()))
            .bind(("note_id", note_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        for bookmark in bookmarks {
            self.surreal
                .delete(("bookmark", bookmark.id.to_string()))
                .await
                .map_err(|e| AppError::Database(e))?;
        }

        info!("Deleted bookmark for user {} note {}", user_id, note_id);

        Ok(())
    }

    /// Get user's bookmarks with pagination
    pub async fn get_bookmarks(
        &self,
        user_id: &ActorId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Bookmark>> {
        let bookmarks: Vec<Bookmark> = self
            .surreal
            .query("SELECT * FROM bookmark WHERE user_id = $user_id ORDER BY created_at DESC LIMIT $limit START $offset")
            .bind(("user_id", user_id.to_string()))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(bookmarks)
    }

    /// Check if a note is bookmarked by user
    pub async fn is_bookmarked(
        &self,
        user_id: &ActorId,
        note_id: &NoteId,
    ) -> Result<bool> {
        let count: Option<i64> = self
            .surreal
            .query("SELECT count() FROM bookmark WHERE user_id = $user_id AND note_id = $note_id GROUP BY count()")
            .bind(("user_id", user_id.to_string()))
            .bind(("note_id", note_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?;

        Ok(count.unwrap_or(0) > 0)
    }

    /// Get bookmark count for a note
    pub async fn get_bookmark_count(&self, note_id: &NoteId) -> Result<i64> {
        let count: Option<i64> = self
            .surreal
            .query("SELECT count() FROM bookmark WHERE note_id = $note_id GROUP BY count()")
            .bind(("note_id", note_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?;

        Ok(count.unwrap_or(0))
    }
}
