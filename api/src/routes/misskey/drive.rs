use crate::state::AppState;
use axum::{
    Extension, Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use mithic_core::models::file::DriveFile;
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{
    create_drive_file, delete_drive_file, get_drive_file, get_drive_file_by_hash, rows_to,
};
use object_store::ObjectStoreExt;
use object_store::path::Path as ObjectPath;
use serde::Deserialize;
use sha2::Digest;
use shared::MediaAttachment;

pub fn file_to_dto(f: &DriveFile) -> MediaAttachment {
    MediaAttachment {
        id: f.id.to_string(),
        url: f.url.clone().unwrap_or_default(),
        preview_url: f.thumbnail_url.clone(),
        media_type: f.mime_type.clone(),
        alt: None,
        is_sensitive: false,
    }
}

pub async fn upload_file(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<MediaAttachment>> {
    let owner_id = auth.user_id;

    let mut name = "file".to_string();
    let mut mime_type = "application/octet-stream".to_string();
    let mut data = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" {
            name = field.file_name().unwrap_or("file").to_string();
            mime_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            data = field
                .bytes()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .to_vec();
        }
    }

    if data.is_empty() {
        return Err(AppError::Validation(
            "No file uploaded or file empty".to_string(),
        ));
    }

    let size = data.len() as i64;
    let hash = hex::encode(sha2::Sha256::digest(&data));

    let file_url = match state.config().storage_type.as_str() {
        "s3" | "minio" | "r2" => {
            if let Some(ref public_url) = state.config().storage_s3_public_url {
                format!("{}/{}", public_url, hash)
            } else {
                format!("{}/uploads/{}", state.config().instance_url, hash)
            }
        }
        "gcs" => {
            if let Some(ref public_url) = state.config().storage_gcs_public_url {
                format!("{}/{}", public_url, hash)
            } else {
                format!("{}/uploads/{}", state.config().instance_url, hash)
            }
        }
        _ => format!("{}/uploads/{}", state.config().instance_url, hash),
    };

    let drive_file = DriveFile::new(name, mime_type, size, owner_id, hash.to_string(), hash);

    let mut drive_file = drive_file;
    drive_file.url = Some(file_url.clone());
    drive_file.thumbnail_url = Some(file_url.clone());

    create_drive_file(state.surreal(), &drive_file)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // object_store を使ってファイルを保存
    let object_path = ObjectPath::from(drive_file.path.as_str());
    state
        .storage()
        .put(&object_path, data.into())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save file: {e}")))?;

    Ok(Json(file_to_dto(&drive_file)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIdRequest {
    pub file_id: String,
}

pub async fn show(
    State(state): State<AppState>,
    Json(request): Json<FileIdRequest>,
) -> Result<Json<MediaAttachment>> {
    let file_id = request
        .file_id
        .parse::<mithic_core::models::file::FileId>()
        .map_err(|_| AppError::Validation("Invalid file id".to_string()))?;

    let file = get_drive_file(state.surreal(), &file_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    Ok(Json(file_to_dto(&file)))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<FileIdRequest>,
) -> Result<StatusCode> {
    let file_id = request
        .file_id
        .parse::<mithic_core::models::file::FileId>()
        .map_err(|_| AppError::Validation("Invalid file id".to_string()))?;

    let file = get_drive_file(state.surreal(), &file_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    if file.owner_id != auth.user_id {
        return Err(AppError::Forbidden("You do not own this file".to_string()));
    }

    delete_drive_file(state.surreal(), &file_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn serve_upload(
    State(state): State<AppState>,
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let file_meta = get_drive_file_by_hash(state.surreal(), &hash)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mime = file_meta
        .map(|f| f.mime_type)
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let object_path = ObjectPath::from(hash.as_str());
    let get_result = state
        .storage()
        .get(&object_path)
        .await
        .map_err(|e| AppError::NotFound(format!("File not found: {e}")))?;

    let bytes = get_result
        .bytes()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read file: {e}")))?;

    let response = axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, mime)
        .body(axum::body::Body::from(bytes))
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(response)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFindRequest {
    pub name: Option<String>,
    pub folder_id: Option<String>,
    pub mime_type: Option<String>,
    pub limit: Option<u64>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

pub async fn find(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<DriveFindRequest>,
) -> Result<Json<Vec<MediaAttachment>>> {
    let limit = request.limit.unwrap_or(10).min(100);

    let mut query = format!(
        "SELECT * FROM drive_file WHERE user_id = type::record('user', $user_id)"
    );
    if let Some(ref name) = request.name {
        query.push_str(&format!(" AND name CONTAINS '{}'", name));
    }
    if let Some(ref mime) = request.mime_type {
        query.push_str(&format!(" AND mime_type CONTAINS '{}'", mime));
    }
    query.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));

    let mut response = state
        .surreal()
        .query(&query)
        .bind(("user_id", auth.user_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<surrealdb::types::Value> = response
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let files: Vec<DriveFile> = rows_to(rows).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(files.iter().map(file_to_dto).collect()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFromUrlRequest {
    pub url: String,
    pub name: Option<String>,
}

pub async fn upload_from_url(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<UploadFromUrlRequest>,
) -> Result<Json<MediaAttachment>> {
    let owner_id = auth.user_id;

    let response = state
        .http_client()
        .get(&request.url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to download: {e}")))?;

    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let data = response
        .bytes()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read response: {e}")))?
        .to_vec();

    if data.is_empty() {
        return Err(AppError::Validation("Downloaded file is empty".to_string()));
    }

    let name = request
        .name
        .unwrap_or_else(|| {
            request
                .url
                .split('/')
                .last()
                .unwrap_or("download")
                .to_string()
        });

    let size = data.len() as i64;
    let hash = hex::encode(sha2::Sha256::digest(&data));

    let file_url = format!("{}/uploads/{}", state.config().instance_url, hash);

    let drive_file = DriveFile::new(name, content_type, size, owner_id, hash.to_string(), hash);
    let mut drive_file = drive_file;
    drive_file.url = Some(file_url.clone());
    drive_file.thumbnail_url = Some(file_url.clone());

    create_drive_file(state.surreal(), &drive_file)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let object_path = ObjectPath::from(drive_file.path.as_str());
    state
        .storage()
        .put(&object_path, data.into())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save file: {e}")))?;

    Ok(Json(file_to_dto(&drive_file)))
}

pub async fn attached_notes(
    State(state): State<AppState>,
    Json(request): Json<FileIdRequest>,
) -> Result<Json<Vec<shared::Note>>> {
    let file_id = request
        .file_id
        .parse::<mithic_core::models::file::FileId>()
        .map_err(|_| AppError::Validation("Invalid file id".to_string()))?;

    let notes = state
        .surreal()
        .query("SELECT * FROM note WHERE file_ids CONTAINS $file_id ORDER BY created_at DESC LIMIT 50")
        .bind(("file_id", file_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(Vec::new())) // TODO: Phase 5 — ノート取得とDTO変換
}
