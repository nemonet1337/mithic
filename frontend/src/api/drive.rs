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

pub async fn find(
    token: &str,
    name: Option<String>,
    mime_type: Option<String>,
    limit: Option<u64>,
) -> Result<Vec<DriveFileResponse>, ApiError> {
    let mut path = "drive/files?limit=".to_string();
    path.push_str(&limit.unwrap_or(20).to_string());
    if let Some(n) = name {
        path.push_str("&name=");
        path.push_str(&urlencoding_loose(&n));
    }
    if let Some(m) = mime_type {
        path.push_str("&mimeType=");
        path.push_str(&urlencoding_loose(&m));
    }
    request::<Vec<DriveFileResponse>, ()>("GET", &path, Some(token), None).await
}

pub async fn delete(token: &str, file_id: &str) -> Result<(), ApiError> {
    request::<(), ()>(
        "DELETE",
        &format!("drive/files/{file_id}"),
        Some(token),
        None,
    )
    .await
}

fn urlencoding_loose(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
