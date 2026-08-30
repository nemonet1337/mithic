use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

pub type FileId = Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum FileType {
    Image,
    Video,
    Audio,
    #[default]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFile {
    pub id: FileId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub name: String,
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
            id: FileId::generate(),
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
