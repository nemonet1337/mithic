use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

pub type InstanceConfigId = Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RegistrationMode { Open, Invite, Closed }

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfig {
    pub id: InstanceConfigId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub registration_mode: RegistrationMode,
    pub maintenance_mode: bool,
    #[validate(length(min = 1, max = 256))]
    pub name: String,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
    pub max_file_size: i64,
    pub max_note_length: i32,
    pub email_verification_required: bool,
    pub captcha_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceConfigRequest {
    pub registration_mode: Option<RegistrationMode>,
    pub maintenance_mode: Option<bool>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_file_size: Option<i64>,
    pub max_note_length: Option<i32>,
    pub email_verification_required: Option<bool>,
    pub captcha_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigResponse {
    pub id: InstanceConfigId,
    pub registration_mode: String,
    pub maintenance_mode: bool,
    pub name: String,
    pub description: Option<String>,
    pub max_file_size: i64,
    pub max_note_length: i32,
    pub email_verification_required: bool,
    pub captcha_enabled: bool,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<InstanceConfig> for InstanceConfigResponse {
    fn from(config: InstanceConfig) -> Self {
        Self {
            id: config.id,
            registration_mode: match config.registration_mode {
                RegistrationMode::Open => "open".to_string(),
                RegistrationMode::Invite => "invite".to_string(),
                RegistrationMode::Closed => "closed".to_string(),
            },
            maintenance_mode: config.maintenance_mode,
            name: config.name,
            description: config.description,
            max_file_size: config.max_file_size,
            max_note_length: config.max_note_length,
            email_verification_required: config.email_verification_required,
            captcha_enabled: config.captcha_enabled,
            updated_at: config.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FederatedInstance {
    pub id: Ulid,
    pub host: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub software_name: Option<String>,
    pub software_version: Option<String>,
    pub icon_url: Option<String>,
    pub favicon_url: Option<String>,
    pub theme_color: Option<String>,
    pub is_blocked: bool,
    pub is_not_responding: bool,
    pub is_suspended: bool,
    pub first_retrieved_at: DateTime<Utc>,
    pub last_retrieved_at: DateTime<Utc>,
    pub info_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FederatedInstanceResponse {
    pub id: String,
    pub host: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub software_name: Option<String>,
    pub software_version: Option<String>,
    pub icon_url: Option<String>,
    pub favicon_url: Option<String>,
    pub theme_color: Option<String>,
    pub is_blocked: bool,
    pub is_not_responding: bool,
    pub is_suspended: bool,
    pub first_retrieved_at: String,
    pub last_retrieved_at: String,
    pub info_updated_at: Option<String>,
}

impl From<FederatedInstance> for FederatedInstanceResponse {
    fn from(instance: FederatedInstance) -> Self {
        Self {
            id: instance.id.to_string(),
            host: instance.host,
            name: instance.name,
            description: instance.description,
            software_name: instance.software_name,
            software_version: instance.software_version,
            icon_url: instance.icon_url,
            favicon_url: instance.favicon_url,
            theme_color: instance.theme_color,
            is_blocked: instance.is_blocked,
            is_not_responding: instance.is_not_responding,
            is_suspended: instance.is_suspended,
            first_retrieved_at: instance.first_retrieved_at.to_rfc3339(),
            last_retrieved_at: instance.last_retrieved_at.to_rfc3339(),
            info_updated_at: instance.info_updated_at.map(|d| d.to_rfc3339()),
        }
    }
}
