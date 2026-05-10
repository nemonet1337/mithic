//! Instance configuration service
//!
//! Manages instance-wide settings and configuration.

use tracing::info;

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{InstanceConfig, InstanceConfigId, RegistrationMode, UpdateInstanceConfigRequest},
};

/// Instance configuration service
#[derive(Debug, Clone)]
pub struct InstanceService {
    surreal: SurrealClient,
    dragonfly: DragonflyClient,
}

impl InstanceService {
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal,
            dragonfly,
        }
    }

    /// Get instance configuration
    pub async fn get_config(&self) -> Result<InstanceConfig> {
        let configs: Vec<InstanceConfig> = self
            .surreal
            .query("SELECT * FROM instance_config LIMIT 1")
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        if let Some(config) = configs.first() {
            Ok(config.clone())
        } else {
            // Create default config if none exists
            let default = self.create_default_config().await?;
            Ok(default)
        }
    }

    /// Update instance configuration
    pub async fn update_config(&self, req: UpdateInstanceConfigRequest) -> Result<InstanceConfig> {
        let config = self.get_config().await?;

        let mut updates = Vec::new();

        if let Some(registration_mode) = req.registration_mode {
            updates.push(format!("registration_mode = '{}'", match registration_mode {
                RegistrationMode::Open => "open",
                RegistrationMode::Invite => "invite",
                RegistrationMode::Closed => "closed",
            }));
        }

        if let Some(maintenance_mode) = req.maintenance_mode {
            updates.push(format!("maintenance_mode = {}", maintenance_mode));
        }

        if let Some(name) = req.name {
            updates.push(format!("name = '{}'", name.replace("'", "''")));
        }

        if let Some(description) = req.description {
            updates.push(format!("description = '{}'", description.replace("'", "''")));
        }

        if let Some(max_file_size) = req.max_file_size {
            updates.push(format!("max_file_size = {}", max_file_size));
        }

        if let Some(max_note_length) = req.max_note_length {
            updates.push(format!("max_note_length = {}", max_note_length));
        }

        if let Some(email_verification_required) = req.email_verification_required {
            updates.push(format!("email_verification_required = {}", email_verification_required));
        }

        if let Some(captcha_enabled) = req.captcha_enabled {
            updates.push(format!("captcha_enabled = {}", captcha_enabled));
        }

        if !updates.is_empty() {
            updates.push("updated_at = time::now()".to_string());
            let query = format!(
                "UPDATE instance_config:{} SET {}",
                config.id,
                updates.join(", ")
            );

            self.surreal
                .query(&query)
                .await
                .map_err(|e| AppError::Database(e))?;

            info!("Updated instance configuration");
        }

        // Fetch updated config
        let updated: Option<InstanceConfig> = self
            .surreal
            .query("SELECT * FROM instance_config WHERE id = $id LIMIT 1")
            .bind(("id", config.id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        updated.ok_or_else(|| AppError::Internal("Failed to fetch updated config".to_string()))
    }

    /// Create default configuration
    async fn create_default_config(&self) -> Result<InstanceConfig> {
        let config = InstanceConfig {
            id: ulid::Ulid::new(),
            created_at: chrono::Utc::now(),
            updated_at: None,
            registration_mode: RegistrationMode::Open,
            maintenance_mode: false,
            name: "Mithic Instance".to_string(),
            description: Some("Welcome to Mithic".to_string()),
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_note_length: 5000,
            email_verification_required: false,
            captcha_enabled: false,
        };

        self.surreal
            .create::<Option<InstanceConfig>>(("instance_config", config.id.to_string()))
            .content(config.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created default instance configuration");

        Ok(config)
    }

    /// Get instance statistics
    pub async fn get_stats(&self) -> Result<InstanceStats> {
        let query = r#"
            SELECT
                (SELECT count() FROM note) AS notes_count,
                (SELECT count() FROM note WHERE actor_id->host = NONE) AS original_notes_count,
                (SELECT count() FROM user) AS users_count,
                (SELECT count() FROM user WHERE host = NONE) AS original_users_count,
                (SELECT count() FROM user WHERE host != NONE) AS remote_users_count
        "#;

        let mut result = self
            .surreal
            .query(query)
            .await
            .map_err(|e| AppError::Database(e))?;

        let stats: Option<InstanceStats> = result.take(0).map_err(|e| AppError::Database(e))?;

        Ok(stats.unwrap_or_else(|| InstanceStats {
            notes_count: 0,
            original_notes_count: 0,
            users_count: 0,
            original_users_count: 0,
            remote_users_count: 0,
            instances: 0,
            drive_usage_local: 0,
            drive_usage_remote: 0,
        }))
    }
}

/// Instance statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStats {
    pub notes_count: i64,
    pub original_notes_count: i64,
    pub users_count: i64,
    pub original_users_count: i64,
    pub remote_users_count: i64,
    pub instances: i64,
    pub drive_usage_local: i64,
    pub drive_usage_remote: i64,
}
