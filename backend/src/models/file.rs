use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

use super::actor::ActorId;

/// ファイルID
pub type FileId = Ulid;

/// ファイルタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileType {
    /// 画像
    Image,
    /// 動画
    Video,
    /// 音声
    Audio,
    /// その他
    Other,
}

impl Default for FileType {
    fn default() -> Self {
        FileType::Other
    }
}

/// ドライブファイルモデル
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DriveFile {
    pub id: FileId,

    pub created_at: DateTime<Utc>,

    pub updated_at: Option<DateTime<Utc>>,

    /// ファイル名
    #[validate(length(min = 1, max = 256))]
    pub name: String,

    /// MIMEタイプ
    #[validate(length(max = 128))]
    pub mime_type: String,

    /// ファイルタイプ
    pub file_type: FileType,

    /// ファイルサイズ（バイト）
    pub size: i64,

    /// 所有者ID
    pub owner_id: ActorId,

    /// ストレージパス（実ファイルの保存場所）
    pub path: String,

    /// サムネイルパス
    pub thumbnail_path: Option<String>,

    /// 公開URL（外部ストレージ使用時）
    pub url: Option<String>,

    /// サムネイルURL
    pub thumbnail_url: Option<String>,

    /// 幅（画像・動画）
    pub width: Option<i32>,

    /// 高さ（画像・動画）
    pub height: Option<i32>,

    /// 期間（動画・音声）
    pub duration: Option<i32>,

    /// ハッシュ（重複チェック用）
    pub hash: String,

    /// 公開設定
    pub is_public: bool,

    /// フォルダID
    pub folder_id: Option<String>,

    /// コメント
    #[validate(length(max = 512))]
    pub comment: Option<String>,
}

impl DriveFile {
    /// 新しいドライブファイルを作成
    pub fn new(
        name: String,
        mime_type: String,
        size: i64,
        owner_id: ActorId,
        path: String,
        hash: String,
    ) -> Self {
        let file_type = Self::detect_file_type(&mime_type);
        let now = Utc::now();

        Self {
            id: FileId::new(),
            created_at: now,
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

    /// MIMEタイプからファイルタイプを判定
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

    /// 画像かどうか
    pub fn is_image(&self) -> bool {
        matches!(self.file_type, FileType::Image)
    }

    /// 動画かどうか
    pub fn is_video(&self) -> bool {
        matches!(self.file_type, FileType::Video)
    }

    /// 音声かどうか
    pub fn is_audio(&self) -> bool {
        matches!(self.file_type, FileType::Audio)
    }

    /// サイズを人間可読な形式で取得
    pub fn human_readable_size(&self) -> String {
        let size = self.size as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// ドライブフォルダモデル
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DriveFolder {
    pub id: String,

    pub created_at: DateTime<Utc>,

    pub updated_at: Option<DateTime<Utc>>,

    /// フォルダ名
    #[validate(length(min = 1, max = 128))]
    pub name: String,

    /// 所有者ID
    pub owner_id: ActorId,

    /// 親フォルダID
    pub parent_id: Option<String>,
}
