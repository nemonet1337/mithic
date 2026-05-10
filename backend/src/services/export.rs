//! Import/Export service for user data
//!
//! Provides data export and import functionality.

use tracing::{error, info};

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{
        ActorId, CreateExportRequest, Export, ExportFormat, ExportScope, ExportStatus,
        Import, ImportRequest, ImportStatus, UserExportData,
    },
};

/// Import/Export service for managing user data
#[derive(Debug, Clone)]
pub struct ExportService {
    surreal: SurrealClient,
    dragonfly: DragonflyClient,
}

impl ExportService {
    /// Create a new export service
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal,
            dragonfly,
        }
    }

    /// Create a new export job
    pub async fn create_export(
        &self,
        user_id: &ActorId,
        request: CreateExportRequest,
    ) -> Result<Export> {
        let export = Export {
            id: ulid::Ulid::new(),
            user_id: user_id.clone(),
            scope: request.scope,
            format: request.format,
            status: ExportStatus::Processing,
            file_url: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };

        self.surreal
            .create::<Option<Export>>(("export", export.id.to_string()))
            .content(export.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created export job {} for user {}", export.id, user_id);

        Ok(export)
    }

    /// Process an export job
    pub async fn process_export(&self, export_id: &ulid::Ulid) -> Result<()> {
        let mut export: Export = self
            .surreal
            .select(("export", export_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("Export not found".to_string()))?;

        // Collect data based on scope
        let user_data = match export.scope {
            ExportScope::All => self.collect_all_data(&export.user_id).await?,
            ExportScope::Notes => {
                let notes = self.collect_notes(&export.user_id).await?;
                let actor = self.get_actor(&export.user_id).await?;
                UserExportData {
                    user_id: export.user_id.clone(),
                    username: actor.username,
                    display_name: actor.display_name,
                    bio: actor.bio,
                    exported_at: chrono::Utc::now(),
                    notes,
                    follows: vec![],
                    clips: vec![],
                    antennas: vec![],
                }
            }
            _ => {
                return Err(AppError::BadRequest("Export scope not implemented yet".to_string()));
            }
        };

        // Serialize to JSON
        let json = serde_json::to_string(&user_data)
            .map_err(|e| AppError::Internal(format!("Failed to serialize data: {}", e)))?;

        // For now, just store the data in the export record
        // In production, this would upload to object storage
        export.file_url = Some(format!("data:application/json;base64,{}", base64::encode(&json)));
        export.status = ExportStatus::Completed;
        export.completed_at = Some(chrono::Utc::now());

        self.surreal
            .update::<Option<Export>>(("export", export_id.to_string()))
            .content(export.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Completed export job {}", export_id);

        Ok(())
    }

    /// Get user's exports
    pub async fn get_user_exports(&self, user_id: &ActorId) -> Result<Vec<Export>> {
        let exports: Vec<Export> = self
            .surreal
            .query("SELECT * FROM export WHERE user_id = $user_id ORDER BY created_at DESC")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(exports)
    }

    /// Get a specific export
    pub async fn get_export(&self, export_id: &ulid::Ulid) -> Result<Export> {
        let export: Option<Export> = self
            .surreal
            .select(("export", export_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        export.ok_or_else(|| AppError::NotFound("Export not found".to_string()))
    }

    /// Delete an export
    pub async fn delete_export(&self, export_id: &ulid::Ulid) -> Result<()> {
        self.surreal
            .delete(("export", export_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Deleted export {}", export_id);

        Ok(())
    }

    /// Create a new import job
    pub async fn create_import(
        &self,
        user_id: &ActorId,
        request: ImportRequest,
    ) -> Result<Import> {
        let import = Import {
            id: ulid::Ulid::new(),
            user_id: user_id.clone(),
            file_id: request.file_id,
            overwrite: request.overwrite,
            status: ImportStatus::Processing,
            imported_notes: 0,
            imported_follows: 0,
            imported_clips: 0,
            imported_antennas: 0,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };

        self.surreal
            .create::<Option<Import>>(("import", import.id.to_string()))
            .content(import.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created import job {} for user {}", import.id, user_id);

        Ok(import)
    }

    /// Process an import job
    pub async fn process_import(&self, import_id: &ulid::Ulid) -> Result<()> {
        let mut import: Import = self
            .surreal
            .select(("import", import_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("Import not found".to_string()))?;

        // Get the file content
        let file: Option<crate::models::DriveFile> = self
            .surreal
            .select(("drive_file", import.file_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let file = file.ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

        // In production, this would download and parse the file
        // For now, we'll mark as completed
        import.status = ImportStatus::Completed;
        import.completed_at = Some(chrono::Utc::now());

        self.surreal
            .update::<Option<Import>>(("import", import_id.to_string()))
            .content(import.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Completed import job {}", import_id);

        Ok(())
    }

    /// Get user's imports
    pub async fn get_user_imports(&self, user_id: &ActorId) -> Result<Vec<Import>> {
        let imports: Vec<Import> = self
            .surreal
            .query("SELECT * FROM import WHERE user_id = $user_id ORDER BY created_at DESC")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(imports)
    }

    /// Get a specific import
    pub async fn get_import(&self, import_id: &ulid::Ulid) -> Result<Import> {
        let import: Option<Import> = self
            .surreal
            .select(("import", import_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        import.ok_or_else(|| AppError::NotFound("Import not found".to_string()))
    }

    /// Collect all user data
    async fn collect_all_data(&self, user_id: &ActorId) -> Result<UserExportData> {
        let actor = self.get_actor(user_id).await?;
        let notes = self.collect_notes(user_id).await?;
        let follows = self.collect_follows(user_id).await?;
        let clips = self.collect_clips(user_id).await?;
        let antennas = self.collect_antennas(user_id).await?;

        Ok(UserExportData {
            user_id: user_id.clone(),
            username: actor.username,
            display_name: actor.display_name,
            bio: actor.bio,
            exported_at: chrono::Utc::now(),
            notes,
            follows,
            clips,
            antennas,
        })
    }

    /// Get actor info
    async fn get_actor(&self, user_id: &ActorId) -> Result<crate::models::Actor> {
        let actor: Option<crate::models::Actor> = self
            .surreal
            .select(("user", user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        actor.ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    /// Collect user notes
    async fn collect_notes(&self, user_id: &ActorId) -> Result<Vec<crate::models::ExportedNote>> {
        let notes: Vec<crate::models::Note> = self
            .surreal
            .query("SELECT * FROM note WHERE actor_id = $user_id ORDER BY created_at DESC")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(notes
            .into_iter()
            .map(|note| crate::models::ExportedNote {
                id: note.id,
                content: note.content,
                created_at: note.created_at,
                visibility: format!("{:?}", note.visibility),
                cw: note.cw,
            })
            .collect())
    }

    /// Collect user follows
    async fn collect_follows(&self, user_id: &ActorId) -> Result<Vec<crate::models::ExportedFollow>> {
        let follows: Vec<crate::models::Follow> = self
            .surreal
            .query("SELECT * FROM follow WHERE follower_id = $user_id")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        let mut result = vec![];
        for follow in follows {
            let following: Option<crate::models::Actor> = self
                .surreal
                .select(("user", follow.following_id.to_string()))
                .await
                .map_err(|e| AppError::Database(e))?;

            if let Some(following) = following {
                result.push(crate::models::ExportedFollow {
                    following_id: follow.following_id,
                    following_username: following.username,
                    created_at: follow.created_at,
                });
            }
        }

        Ok(result)
    }

    /// Collect user clips
    async fn collect_clips(&self, user_id: &ActorId) -> Result<Vec<crate::models::ExportedClip>> {
        let clips: Vec<crate::models::Clip> = self
            .surreal
            .query("SELECT * FROM clip WHERE user_id = $user_id")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(clips
            .into_iter()
            .map(|clip| crate::models::ExportedClip {
                name: clip.name,
                description: clip.description,
                is_public: clip.is_public,
                note_ids: clip.note_ids,
                created_at: clip.created_at,
            })
            .collect())
    }

    /// Collect user antennas
    async fn collect_antennas(&self, user_id: &ActorId) -> Result<Vec<crate::models::ExportedAntenna>> {
        let antennas: Vec<crate::models::Antenna> = self
            .surreal
            .query("SELECT * FROM antenna WHERE user_id = $user_id")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(antennas
            .into_iter()
            .map(|antenna| crate::models::ExportedAntenna {
                name: antenna.name,
                keywords: antenna.keywords,
                exclude_keywords: antenna.exclude_keywords,
                source: format!("{:?}", antenna.source),
                case_sensitive: antenna.case_sensitive,
                with_replies: antenna.with_replies,
                with_renotes: antenna.with_renotes,
            })
            .collect())
    }
}
