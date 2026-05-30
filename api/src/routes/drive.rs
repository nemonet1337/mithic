use axum::{Extension, Json, extract::{State, Multipart}, http::StatusCode};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{create_drive_file, get_drive_file, delete_drive_file};
use mithic_core::models::file::DriveFile;
use crate::state::AppState;
use serde::Deserialize;
use shared::MediaAttachment;
use sha2::Digest;

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

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Internal(e.to_string()))? {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" {
            name = field.file_name().unwrap_or("file").to_string();
            mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            data = field.bytes().await.map_err(|e| AppError::Internal(e.to_string()))?.to_vec();
        }
    }

    if data.is_empty() {
        return Err(AppError::Validation("No file uploaded or file empty".to_string()));
    }

    let size = data.len() as i64;
    let hash = hex::encode(sha2::Sha256::digest(&data));

    let file_url = format!("{}/uploads/{}", state.config().instance_url, hash);

    let drive_file = DriveFile::new(
        name,
        mime_type,
        size,
        owner_id,
        format!("{}", hash),
        hash,
    );

    let mut drive_file = drive_file;
    drive_file.url = Some(file_url.clone());
    drive_file.thumbnail_url = Some(file_url.clone());

    create_drive_file(state.surreal(), &drive_file)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let uploads_dir = std::path::Path::new("uploads");
    if !uploads_dir.exists() {
        let _ = std::fs::create_dir_all(uploads_dir);
    }
    let file_path = uploads_dir.join(&drive_file.path);
    let _ = std::fs::write(file_path, data);

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
    let file_id = request.file_id.parse::<mithic_core::models::file::FileId>()
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
    let file_id = request.file_id.parse::<mithic_core::models::file::FileId>()
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
