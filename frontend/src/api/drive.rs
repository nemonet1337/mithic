use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FormData, Headers, Request, RequestInit, RequestMode, Response};

use super::client::{api_base, ApiError, request};

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

/// multipart アップロード（フィールド名 `file`）
pub async fn upload(token: &str, file: &web_sys::File) -> Result<DriveFileResponse, ApiError> {
    let url = format!("{}/drive/files", api_base());

    let form = FormData::new().map_err(|e| ApiError {
        status: 0,
        code: "formdata".into(),
        message: format!("FormData: {e:?}"),
        detail: None,
    })?;

    let name = file.name();
    form.append_with_blob_and_filename("file", file, &name)
        .map_err(|e| ApiError {
            status: 0,
            code: "formdata".into(),
            message: format!("append file: {e:?}"),
            detail: None,
        })?;

    let headers = Headers::new().map_err(|e| ApiError {
        status: 0,
        code: "headers".into(),
        message: format!("{e:?}"),
        detail: None,
    })?;
    headers
        .set("Authorization", &format!("Bearer {token}"))
        .map_err(|e| ApiError {
            status: 0,
            code: "headers".into(),
            message: format!("{e:?}"),
            detail: None,
        })?;
    // Content-Type は付けない（boundary はブラウザが付与）

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::SameOrigin);
    opts.set_headers(&headers);
    opts.set_body(form.as_ref());

    let request = Request::new_with_str_and_init(&url, &opts).map_err(|e| ApiError {
        status: 0,
        code: "request".into(),
        message: format!("{e:?}"),
        detail: None,
    })?;

    let window = web_sys::window().ok_or_else(|| ApiError {
        status: 0,
        code: "window".into(),
        message: "no window".into(),
        detail: None,
    })?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| ApiError {
            status: 0,
            code: "network".into(),
            message: format!("upload fetch failed: {e:?}"),
            detail: None,
        })?;

    let response: Response = resp_value.dyn_into().map_err(|_| ApiError {
        status: 0,
        code: "response".into(),
        message: "invalid response type".into(),
        detail: None,
    })?;

    let status = response.status();
    let text = JsFuture::from(response.text().map_err(|e| ApiError {
        status,
        code: "body".into(),
        message: format!("{e:?}"),
        detail: None,
    })?)
    .await
    .map_err(|e| ApiError {
        status,
        code: "body".into(),
        message: format!("{e:?}"),
        detail: None,
    })?;

    let body = text.as_string().unwrap_or_default();

    if !(200..300).contains(&status) {
        let mut err: ApiError = serde_json::from_str(&body).unwrap_or(ApiError {
            status,
            code: "upload".into(),
            message: format!("HTTP {status}"),
            detail: None,
        });
        if err.status == 0 {
            err.status = status;
        }
        return Err(err);
    }

    serde_json::from_str(&body).map_err(|e| ApiError {
        status: 0,
        code: "deserialize".into(),
        message: e.to_string(),
        detail: None,
    })
}

fn urlencoding_loose(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
