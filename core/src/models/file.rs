use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

use super::actor::ActorId;

pub type FileId = Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileType {
    Image,
    Video,
    Audio,
    Other,
}

impl Default for FileType {
    fn default() -> Self {
        FileType::Other
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DriveFile {
    pub id: FileId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    #[validate(length(min = 1, max = 256))]
    pub name: String,
    #[validate(length(max = 128))]
    pub mime_type: String,
    pub file_type: FileType,
    pub size: i64,
    pub owner_id: ActorId,
    pub path: String,
    pub thumbnail_path: Option<String>,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<i32>,
    pub hash: String,
    pub is_public: bool,
    pub folder_id: Option<String>,
    #[validate(length(max = 512))]
    pub comment: Option<String>,
}

impl DriveFile {
    pub fn new(
        name: String,
        mime_type: String,
        size: i64,
        owner_id: ActorId,
        path: String,
        hash: String,
    ) -> Self {
        let file_type = Self::detect_file_type(&mime_type);
        Self {
            id: FileId::new(),
            created_at: Utc::now(),
            updated_at: None,
            name,
            mime_type,
            file_type,
            size,
            owner_id,
            path,
            thumbnail_path: None,
            url: None,
            thumbnail_url: None,
            width: None,
            height: None,
            duration: None,
            hash,
            is_public: false,
            folder_id: None,
            comment: None,
        }
    }

    fn detect_file_type(mime_type: &str) -> FileType {
        if mime_type.starts_with("image/") {
            FileType::Image
        } else if mime_type.starts_with("video/") {
            FileType::Video
        } else if mime_type.starts_with("audio/") {
            FileType::Audio
        } else {
            FileType::Other
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(self.file_type, FileType::Image)
    }
    pub fn is_video(&self) -> bool {
        matches!(self.file_type, FileType::Video)
    }
    pub fn is_audio(&self) -> bool {
        matches!(self.file_type, FileType::Audio)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DriveFolder {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    #[validate(length(min = 1, max = 128))]
    pub name: String,
    pub owner_id: ActorId,
    pub parent_id: Option<String>,
}
