//! Antenna service for keyword monitoring
//!
//! Provides automatic note collection based on keywords and conditions.

use tracing::{error, info};

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{
        ActorId, Antenna, AntennaId, AntennaNote, AntennaNoteId, AntennaSource, CreateAntennaRequest,
        NoteId, UpdateAntennaRequest,
    },
};

/// Antenna service for managing keyword-based monitoring
#[derive(Debug, Clone)]
pub struct AntennaService {
    surreal: SurrealClient,
    dragonfly: DragonflyClient,
}

impl AntennaService {
    /// Create a new antenna service
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal,
            dragonfly,
        }
    }

    /// Create a new antenna
    pub async fn create_antenna(
        &self,
        user_id: &ActorId,
        request: CreateAntennaRequest,
    ) -> Result<Antenna> {
        let antenna = Antenna {
            id: AntennaId::new(),
            user_id: user_id.clone(),
            name: request.name,
            source: request.source,
            keywords: request.keywords,
            exclude_keywords: request.exclude_keywords,
            users: request.users,
            user_list_id: request.user_list_id,
            case_sensitive: request.case_sensitive,
            with_replies: request.with_replies,
            with_renotes: request.with_renotes,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };

        self.surreal
            .create::<Option<Antenna>>(("antenna", antenna.id.to_string()))
            .content(antenna.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created antenna {} for user {}", antenna.id, user_id);

        Ok(antenna)
    }

    /// Update an antenna
    pub async fn update_antenna(
        &self,
        antenna_id: &AntennaId,
        request: UpdateAntennaRequest,
    ) -> Result<Antenna> {
        let mut antenna: Antenna = self
            .surreal
            .select(("antenna", antenna_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("Antenna not found".to_string()))?;

        if let Some(name) = request.name {
            antenna.name = name;
        }
        if let Some(source) = request.source {
            antenna.source = source;
        }
        if let Some(keywords) = request.keywords {
            antenna.keywords = keywords;
        }
        if let Some(exclude_keywords) = request.exclude_keywords {
            antenna.exclude_keywords = exclude_keywords;
        }
        if let Some(users) = request.users {
            antenna.users = users;
        }
        if let Some(user_list_id) = request.user_list_id {
            antenna.user_list_id = Some(user_list_id);
        }
        if let Some(case_sensitive) = request.case_sensitive {
            antenna.case_sensitive = case_sensitive;
        }
        if let Some(with_replies) = request.with_replies {
            antenna.with_replies = with_replies;
        }
        if let Some(with_renotes) = request.with_renotes {
            antenna.with_renotes = with_renotes;
        }
        antenna.updated_at = Some(chrono::Utc::now());

        self.surreal
            .update::<Option<Antenna>>(("antenna", antenna_id.to_string()))
            .content(antenna.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Updated antenna {}", antenna_id);

        Ok(antenna)
    }

    /// Delete an antenna
    pub async fn delete_antenna(&self, antenna_id: &AntennaId) -> Result<()> {
        self.surreal
            .delete(("antenna", antenna_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        // Delete associated antenna notes
        self.surreal
            .query("DELETE FROM antenna_note WHERE antenna_id = $antenna_id")
            .bind(("antenna_id", antenna_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Deleted antenna {}", antenna_id);

        Ok(())
    }

    /// Get all antennas for a user
    pub async fn get_user_antennas(&self, user_id: &ActorId) -> Result<Vec<Antenna>> {
        let antennas: Vec<Antenna> = self
            .surreal
            .query("SELECT * FROM antenna WHERE user_id = $user_id")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(antennas)
    }

    /// Get a specific antenna
    pub async fn get_antenna(&self, antenna_id: &AntennaId) -> Result<Antenna> {
        let antenna: Option<Antenna> = self
            .surreal
            .select(("antenna", antenna_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        antenna.ok_or_else(|| AppError::NotFound("Antenna not found".to_string()))
    }

    /// Check if a note matches an antenna's criteria
    pub fn note_matches_antenna(&self, antenna: &Antenna, note_text: &str) -> bool {
        // Check exclude keywords first
        for exclude in &antenna.exclude_keywords {
            let text = if antenna.case_sensitive {
                note_text.to_string()
            } else {
                note_text.to_lowercase()
            };
            let pattern = if antenna.case_sensitive {
                exclude.clone()
            } else {
                exclude.to_lowercase()
            };

            if text.contains(&pattern) {
                return false;
            }
        }

        // Check include keywords (OR logic)
        if !antenna.keywords.is_empty() {
            let text = if antenna.case_sensitive {
                note_text.to_string()
            } else {
                note_text.to_lowercase()
            };

            let matches = antenna.keywords.iter().any(|keyword| {
                let pattern = if antenna.case_sensitive {
                    keyword.clone()
                } else {
                    keyword.to_lowercase()
                };
                text.contains(&pattern)
            });

            if !matches {
                return false;
            }
        }

        true
    }

    /// Add a note to an antenna
    pub async fn add_note_to_antenna(
        &self,
        antenna_id: &AntennaId,
        note_id: &NoteId,
    ) -> Result<()> {
        // Check if already added
        let existing: Option<AntennaNote> = self
            .surreal
            .query("SELECT * FROM antenna_note WHERE antenna_id = $antenna_id AND note_id = $note_id LIMIT 1")
            .bind(("antenna_id", antenna_id.to_string()))
            .bind(("note_id", note_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .and_then(|v: Vec<AntennaNote>| v.into_iter().next());

        if existing.is_some() {
            return Ok(());
        }

        let antenna_note = AntennaNote {
            id: AntennaNoteId::new(),
            antenna_id: *antenna_id,
            note_id: *note_id,
            created_at: chrono::Utc::now(),
        };

        self.surreal
            .create::<Option<AntennaNote>>(("antenna_note", antenna_note.id.to_string()))
            .content(antenna_note)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    /// Get notes for an antenna
    pub async fn get_antenna_notes(
        &self,
        antenna_id: &AntennaId,
        limit: u32,
    ) -> Result<Vec<NoteId>> {
        let notes: Vec<AntennaNote> = self
            .surreal
            .query("SELECT * FROM antenna_note WHERE antenna_id = $antenna_id ORDER BY created_at DESC LIMIT $limit")
            .bind(("antenna_id", antenna_id.to_string()))
            .bind(("limit", limit as i64))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(notes.into_iter().map(|n| n.note_id).collect())
    }

    /// Remove a note from an antenna
    pub async fn remove_note_from_antenna(
        &self,
        antenna_id: &AntennaId,
        note_id: &NoteId,
    ) -> Result<()> {
        self.surreal
            .query("DELETE FROM antenna_note WHERE antenna_id = $antenna_id AND note_id = $note_id")
            .bind(("antenna_id", antenna_id.to_string()))
            .bind(("note_id", note_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    /// Process a new note against all antennas (called when a note is created)
    pub async fn process_note(&self, note_id: &NoteId, note_text: &str) -> Result<()> {
        // Get all antennas
        let antennas: Vec<Antenna> = self
            .surreal
            .query("SELECT * FROM antenna")
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        for antenna in antennas {
            if self.note_matches_antenna(&antenna, note_text) {
                if let Err(e) = self.add_note_to_antenna(&antenna.id, note_id).await {
                    error!("Failed to add note {} to antenna {}: {}", note_id, antenna.id, e);
                }
            }
        }

        Ok(())
    }
}
