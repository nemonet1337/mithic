use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

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
) -> Result<Vec<DriveFileResponse>, crate::api::client::ApiError> {
    let client = crate::api::client::get_client();
    let body = DriveFindRequest { name, mime_type, limit };
    let resp = Request::post(&format!("{}/api/drive/files/find", client.base_url))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&body)
        .map_err(|e| crate::api::client::ApiError {
            status: 0,
            message: e.to_string(),
        })?
        .send()
        .await
        .map_err(|e| crate::api::client::ApiError {
            status: 0,
            message: e.to_string(),
        })?;
    if resp.ok() {
        resp.json()
            .await
            .map_err(|e| crate::api::client::ApiError {
                status: 0,
                message: e.to_string(),
            })
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(crate::api::client::ApiError {
            status,
            message: text,
        })
    }
}

pub async fn delete(
    token: &str,
    file_id: &str,
) -> Result<(), crate::api::client::ApiError> {
    let client = crate::api::client::get_client();
    let body = serde_json::json!({ "fileId": file_id });
    let resp = Request::post(&format!("{}/api/drive/files/delete", client.base_url))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&body)
        .map_err(|e| crate::api::client::ApiError {
            status: 0,
            message: e.to_string(),
        })?
        .send()
        .await
        .map_err(|e| crate::api::client::ApiError {
            status: 0,
            message: e.to_string(),
        })?;
    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(crate::api::client::ApiError {
            status,
            message: text,
        })
    }
}
