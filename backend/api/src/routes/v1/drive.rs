//! Drive files: upload / list / show / delete / from-url

use axum::{
    Extension, Json,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
};
use futures_util::TryStreamExt;
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

use crate::dto::drive_file_to_attachment;
use crate::ssrf::{max_fetch_bytes, read_body_limited, validate_public_url};
use crate::state::AppState;

const ALLOWED_MIME_PREFIXES: &[&str] = &["image/", "video/", "audio/"];
const MAX_UPLOAD_BYTES: usize = 32 * 1024 * 1024;

pub fn file_to_dto(f: &DriveFile) -> MediaAttachment {
    drive_file_to_attachment(f)
}

fn detect_mime(data: &[u8], claimed: &str) -> Result<String> {
    let detected = if data.len() >= 3 && data[0] == 0xff && data[1] == 0xd8 && data[2] == 0xff {
        Some("image/jpeg")
    } else if data.len() >= 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
        Some("image/png")
    } else if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
        Some("image/gif")
    } else if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        Some("image/webp")
    } else if data.len() >= 12 && &data[4..8] == b"ftyp" {
        Some("video/mp4")
    } else if data.len() >= 4 && &data[0..4] == b"OggS" {
        Some("audio/ogg")
    } else if (data.len() >= 3 && &data[0..3] == b"ID3")
        || (data.len() >= 2 && data[0] == 0xff && (data[1] & 0xe0) == 0xe0)
    {
        Some("audio/mpeg")
    } else {
        None
    };

    let mime = detected.unwrap_or(claimed).to_string();
    let allowed = ALLOWED_MIME_PREFIXES.iter().any(|p| mime.starts_with(p));
    if !allowed {
        return Err(AppError::Validation(format!(
            "MIME type not allowed: {mime}"
        )));
    }
    Ok(mime)
}

fn is_inline_mime(mime: &str) -> bool {
    mime.starts_with("image/") || mime.starts_with("video/") || mime.starts_with("audio/")
}

fn public_url(state: &AppState, object_key: &str) -> String {
    match state.config().storage_type.as_str() {
        "s3" | "minio" | "r2" => {
            if let Some(ref public_url) = state.config().storage_s3_public_url {
                format!("{public_url}/{object_key}")
            } else {
                format!("{}/uploads/{object_key}", state.config().instance_url)
            }
        }
        "gcs" => {
            if let Some(ref public_url) = state.config().storage_gcs_public_url {
                format!("{public_url}/{object_key}")
            } else {
                format!("{}/uploads/{object_key}", state.config().instance_url)
            }
        }
        _ => format!("{}/uploads/{object_key}", state.config().instance_url),
    }
}

/// Resize to max 400px and encode as WebP. Returns (bytes, original_w, original_h).
fn make_thumbnail_webp(data: &[u8]) -> Option<(Vec<u8>, i32, i32)> {
    use std::io::Cursor;

    let img = image::load_from_memory(data).ok()?;
    let (w, h) = (img.width() as i32, img.height() as i32);
    let thumb = img.thumbnail(400, 400);
    let mut buf = Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::WebP).ok()?;
    Some((buf.into_inner(), w, h))
}

async fn store_file(
    state: &AppState,
    owner_id: mithic_core::models::actor::ActorId,
    name: String,
    mime_type: String,
    data: Vec<u8>,
) -> Result<DriveFile> {
    let size = data.len() as i64;
    let hash = hex::encode(sha2::Sha256::digest(&data));
    let file_url = public_url(state, &hash);

    let mut drive_file = DriveFile::new(
        name,
        mime_type.clone(),
        size,
        owner_id,
        hash.clone(),
        hash.clone(),
    );
    drive_file.url = Some(file_url.clone());
    drive_file.thumbnail_url = Some(file_url);

    // Generate thumbnail before DB write so URLs match what is stored
    let mut thumb_store: Option<(String, Vec<u8>)> = None;
    if mime_type.starts_with("image/") {
        if let Some((bytes, w, h)) = make_thumbnail_webp(&data) {
            let thumb_key = format!("{hash}.thumb");
            drive_file.width = Some(w);
            drive_file.height = Some(h);
            drive_file.thumbnail_path = Some(thumb_key.clone());
            drive_file.thumbnail_url = Some(public_url(state, &thumb_key));
            thumb_store = Some((thumb_key, bytes));
        }
    }

    // Persist original first
    let object_path = ObjectPath::from(hash.as_str());
    state
        .storage()
        .put(&object_path, data.into())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save file: {e}")))?;

    if let Some((thumb_key, bytes)) = thumb_store {
        let thumb_path = ObjectPath::from(thumb_key.as_str());
        if let Err(e) = state.storage().put(&thumb_path, bytes.into()).await {
            tracing::warn!("Failed to save thumbnail {thumb_key}: {e}");
            drive_file.thumbnail_path = None;
            drive_file.thumbnail_url = drive_file.url.clone();
        }
    }

    create_drive_file(state.surreal(), &drive_file)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(drive_file)
}

pub async fn upload_file(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<MediaAttachment>> {
    let mut name = "file".to_string();
    let mut claimed_mime = "application/octet-stream".to_string();
    let mut data = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" {
            name = field.file_name().unwrap_or("file").to_string();
            claimed_mime = field
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
    if data.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::Validation(format!(
            "File too large (max {MAX_UPLOAD_BYTES} bytes)"
        )));
    }

    let mime_type = detect_mime(&data, &claimed_mime)?;
    let drive_file = store_file(&state, auth.user_id, name, mime_type, data).await?;
    Ok(Json(file_to_dto(&drive_file)))
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MediaAttachment>> {
    let file_id = id
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
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let file_id = id
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFindQuery {
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub limit: Option<u64>,
}

pub async fn find(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(request): Query<DriveFindQuery>,
) -> Result<Json<Vec<MediaAttachment>>> {
    let limit = request.limit.unwrap_or(10).min(100);

    let mut query =
        String::from("SELECT * FROM drive_file WHERE user_id = type::record('user', $user_id)");
    if request.name.is_some() {
        query.push_str(" AND name CONTAINS $name");
    }
    if request.mime_type.is_some() {
        query.push_str(" AND mime_type CONTAINS $mime");
    }
    query.push_str(" ORDER BY created_at DESC LIMIT $limit");

    let mut q = state
        .surreal()
        .query(&query)
        .bind(("user_id", auth.user_id.to_string()))
        .bind(("limit", limit));

    if let Some(ref name) = request.name {
        q = q.bind(("name", name.clone()));
    }
    if let Some(ref mime) = request.mime_type {
        q = q.bind(("mime", mime.clone()));
    }

    let mut response = q.await.map_err(|e| AppError::Internal(e.to_string()))?;
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
    validate_public_url(&request.url)?;

    let response = state
        .http_client()
        .get(&request.url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to download: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .split(';')
        .next()
        .unwrap_or("application/octet-stream")
        .trim()
        .to_string();

    let data = read_body_limited(response, max_fetch_bytes()).await?;
    if data.is_empty() {
        return Err(AppError::Validation("Downloaded file is empty".to_string()));
    }

    let mime_type = detect_mime(&data, &content_type)?;
    let name = request.name.unwrap_or_else(|| {
        request
            .url
            .split('/')
            .next_back()
            .unwrap_or("download")
            .to_string()
    });

    let drive_file = store_file(&state, auth.user_id, name, mime_type, data).await?;
    Ok(Json(file_to_dto(&drive_file)))
}

pub async fn attached_notes(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<Vec<shared::Note>>> {
    // ponytail: 添付ノート逆引きは後で実装
    Ok(Json(Vec::new()))
}

pub async fn serve_upload(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    // `{sha256}.thumb` is a generated WebP preview; originals use bare sha256 hex.
    let is_thumb = hash.ends_with(".thumb");
    let mime = if is_thumb {
        "image/webp".to_string()
    } else {
        let base = hash.strip_suffix(".thumb").unwrap_or(&hash);
        let file_meta = get_drive_file_by_hash(state.surreal(), base)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        file_meta
            .map(|f| f.mime_type)
            .unwrap_or_else(|| "application/octet-stream".to_string())
    };

    let object_path = ObjectPath::from(hash.as_str());
    let get_result = state
        .storage()
        .get(&object_path)
        .await
        .map_err(|e| AppError::NotFound(format!("File not found: {e}")))?;

    let stream = get_result
        .into_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
    let body = axum::body::Body::from_stream(stream);

    let disposition = if is_inline_mime(&mime) {
        "inline"
    } else {
        "attachment"
    };

    let response = axum::response::Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(body)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(response)
}
