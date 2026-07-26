use serde::{Deserialize, Serialize};

use super::client::{ApiError, request};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFileResponse {
    pub id: String,
    pub url: String,
    pub preview_url: Option<String>,
    pub media_type: String,
    pub alt: Option<String>,
    pub is_sensitive: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DriveFindRequest {
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub limit: Option<u64>,
}

pub async fn find(
    token: &str,
    name: Option<String>,
    mime_type: Option<String>,
    limit: Option<u64>,
) -> Result<Vec<DriveFileResponse>, ApiError> {
    let body = DriveFindRequest { name, mime_type, limit };
    request::<Vec<DriveFileResponse>, _>(
        "POST",
        "drive/files/find",
        Some(token),
        Some(&body),
    )
    .await
}

pub async fn delete(
    token: &str,
    file_id: &str,
) -> Result<(), ApiError> {
    let body = serde_json::json!({ "fileId": file_id });
    request::<(), serde_json::Value>(
        "POST",
        "drive/files/delete",
        Some(token),
        Some(&body),
    )
    .await
}
