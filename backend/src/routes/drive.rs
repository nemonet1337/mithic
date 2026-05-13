use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use tracing::{error, info};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{DriveFile, FileType},
    state::{AppState, AuthUser},
};

#[derive(Debug, Deserialize)]
pub struct FindFilesQuery {
    pub name: Option<String>,
    pub folder_id: Option<String>,
    pub mime_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct UploadFromUrlRequest {
    pub url: String,
    pub folder_id: Option<String>,
    pub name: Option<String>,
    pub is_sensitive: Option<bool>,
    pub comment: Option<String>,
    pub marker: Option<String>,
    pub force: Option<bool>,
}

/// ファイルレスポンス
#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDriveFileRequest {
    pub name: Option<String>,
    pub folder_id: Option<String>,
}

impl From<&DriveFile> for FileResponse {
    fn from(file: &DriveFile) -> Self {
        Self {
            id: file.id.to_string(),
            name: file.name.clone(),
            mime_type: file.mime_type.clone(),
            size: file.size,
            url: file.url.clone(),
            thumbnail_url: file.thumbnail_url.clone(),
            width: file.width,
            height: file.height,
        }
    }
}

/// ファイルアップロード
pub async fn upload_file(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<FileResponse>> {
    // フィールドを取得
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read multipart: {}", e)))?
        .ok_or_else(|| AppError::Validation("No file provided".to_string()))?;

    let file_name = field.file_name().unwrap_or("unknown").to_string();
    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    // データを読み込み
    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read file: {}", e)))?;

    let size = data.len() as i64;

    // サイズ制限チェック（100MB）
    if size > 100 * 1024 * 1024 {
        return Err(AppError::Validation("File too large (max 100MB)".to_string()));
    }

    // SHA256ハッシュ計算
    let hash = format!("{:x}", Sha256::digest(&data));

    // 既存ファイルチェック（重複排除）
    let check_query = r#"
        SELECT * FROM drive_file WHERE hash = $hash AND owner_id = $owner_id
    "#;
    let mut check_result = state
        .surreal()
        .query(check_query)
        .bind(("hash", hash.clone()))
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let existing: Option<DriveFile> = check_result.take(0).ok().flatten();
    if let Some(file) = existing {
        info!("Duplicate file detected, returning existing file: {}", file.id);
        return Ok(Json(FileResponse::from(&file)));
    }

    // 保存先パスを決定
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
    let file_id = Ulid::new();
    let ext = std::path::Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let save_name = format!("{}.{}", file_id, ext);
    let file_path = std::path::Path::new(&upload_dir).join(&save_name);

    // ディレクトリ作成
    std::fs::create_dir_all(&upload_dir).map_err(|e| {
        error!("Failed to create upload directory: {}", e);
        AppError::Internal(format!("Failed to create upload directory: {}", e))
    })?;

    // ファイル保存
    let mut file = std::fs::File::create(&file_path).map_err(|e| {
        error!("Failed to create file: {}", e);
        AppError::Internal(format!("Failed to create file: {}", e))
    })?;
    file.write_all(&data).map_err(|e| {
        error!("Failed to write file: {}", e);
        AppError::Internal(format!("Failed to write file: {}", e))
    })?;

    // 画像の場合はサイズを取得
    let (width, height) = if content_type.starts_with("image/") {
        get_image_dimensions(&data)
    } else {
        (None, None)
    };

    // ドライブファイルレコード作成
    let path_str = file_path.to_string_lossy().to_string();
    let mut drive_file = DriveFile::new(
        file_name,
        content_type,
        size,
        auth_user.user_id,
        path_str,
        hash,
    );
    drive_file.width = width;
    drive_file.height = height;

    // DBに保存
    let created: DriveFile = state
        .surreal()
        .create(("drive_file", drive_file.id.to_string()))
        .content(drive_file)
        .await
        .map_err(|e| {
            error!("Failed to create drive file record: {}", e);
            AppError::Database(e)
        })?;

    info!(
        "File uploaded: {} ({} bytes) by user {}",
        created.name, created.size, auth_user.user_id
    );

    Ok(Json(FileResponse::from(&created)))
}

/// 画像サイズを取得
fn get_image_dimensions(data: &[u8]) -> (Option<i32>, Option<i32>) {
    match image::load_from_memory(data) {
        Ok(img) => {
            let (w, h) = img.dimensions();
            (Some(w as i32), Some(h as i32))
        }
        Err(e) => {
            tracing::warn!("Failed to get image dimensions: {}", e);
            (None, None)
        }
    }
}

/// ユーザーのドライブファイル一覧取得
pub async fn get_drive_files(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<Vec<FileResponse>>> {
    let query = r#"
        SELECT * FROM drive_file WHERE owner_id = $owner_id ORDER BY created_at DESC LIMIT 100
    "#;
    let mut result = state
        .surreal()
        .query(query)
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let files: Vec<DriveFile> = result.take(0).unwrap_or_default();
    let responses: Vec<FileResponse> = files.iter().map(FileResponse::from).collect();

    Ok(Json(responses))
}

/// ドライブファイル詳細取得
pub async fn get_drive_file(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<FileResponse>> {
    let file_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid file ID".to_string()))?;

    let query = r#"
        SELECT * FROM drive_file WHERE id = $id AND owner_id = $owner_id
    "#;
    let mut result = state
        .surreal()
        .query(query)
        .bind(("id", file_id.to_string()))
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let file: Option<DriveFile> = result.take(0).ok().flatten();
    let file = file.ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    Ok(Json(FileResponse::from(&file)))
}

/// ドライブファイル更新
pub async fn update_drive_file(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDriveFileRequest>,
) -> Result<Json<FileResponse>> {
    let file_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid file ID".to_string()))?;

    // ファイルの存在確認
    let check_query = r#"
        SELECT * FROM drive_file WHERE id = $id AND owner_id = $owner_id
    "#;
    let mut check_result = state
        .surreal()
        .query(check_query)
        .bind(("id", file_id.to_string()))
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let file: Option<DriveFile> = check_result.take(0).ok().flatten();
    let file = file.ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    // 更新クエリ構築
    let mut update_parts = Vec::new();
    if req.name.is_some() {
        update_parts.push("name = $name");
    }
    if req.folder_id.is_some() {
        update_parts.push("folder_id = $folder_id");
    }

    if update_parts.is_empty() {
        return Ok(Json(FileResponse::from(&file)));
    }

    let update_query = format!(
        "UPDATE drive_file:{} SET {}",
        file_id.to_string(),
        update_parts.join(", ")
    );

    let mut surreal_query = state.surreal().query(&update_query);
    if let Some(name) = req.name {
        surreal_query = surreal_query.bind(("name", name));
    }
    if let Some(folder_id) = req.folder_id {
        surreal_query = surreal_query.bind(("folder_id", folder_id));
    }

    surreal_query.await.map_err(|e| AppError::Database(e))?;

    // 更新後のファイルを取得
    let updated: Option<DriveFile> = state
        .surreal()
        .query("SELECT * FROM drive_file WHERE id = $id")
        .bind(("id", file_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    let updated = updated.ok_or_else(|| AppError::Internal("Failed to fetch updated file".to_string()))?;

    Ok(Json(FileResponse::from(&updated)))
}

/// ドライブファイル削除
pub async fn delete_drive_file(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let file_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid file ID".to_string()))?;

    // ファイルの存在確認
    let check_query = r#"
        SELECT * FROM drive_file WHERE id = $id AND owner_id = $owner_id
    "#;
    let mut check_result = state
        .surreal()
        .query(check_query)
        .bind(("id", file_id.to_string()))
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let file: Option<DriveFile> = check_result.take(0).ok().flatten();
    let file = file.ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    // ファイルを削除
    let delete_query = r#"
        DELETE drive_file:$id
    "#;

    state
        .surreal()
        .query(delete_query)
        .bind(("id", file_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    // 実ファイルの削除
    if let Err(e) = std::fs::remove_file(&file.path) {
        error!("Failed to delete file from disk: {}", e);
    }

    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// ドライブ使用量
pub async fn get_drive_usage(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<serde_json::Value>> {
    let query = r#"
        SELECT math::sum(size) as total_size FROM drive_file WHERE owner_id = $owner_id
    "#;
    let mut result = state
        .surreal()
        .query(query)
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let size_result: Option<serde_json::Value> = result.take(0).ok().flatten();
    let total_size = size_result
        .and_then(|v| v.get("total_size"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // ファイル数
    let count_query = r#"
        SELECT count() FROM drive_file WHERE owner_id = $owner_id
    "#;
    let mut count_result = state
        .surreal()
        .query(count_query)
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let count: Option<i64> = count_result.take(0).ok().flatten();
    let file_count = count.unwrap_or(0);

    Ok(Json(serde_json::json!({
        "usage": total_size,
        "capacity": 1024 * 1024 * 1024 * 10, // 10GB
        "file_count": file_count,
    })))
}

/// ファイルを参照しているノートを取得
pub async fn get_attached_notes(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let file_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid file ID".to_string()))?;

    // Verify ownership
    let file: Option<DriveFile> = state.surreal()
        .query("SELECT * FROM drive_file WHERE id = $id AND owner_id = $owner_id")
        .bind(("id", file_id.to_string()))
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    file.ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let notes: Vec<serde_json::Value> = state.surreal()
        .query("SELECT * FROM note WHERE $file_id INSIDE file_ids")
        .bind(("file_id", file_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    Ok(Json(notes))
}

/// MD5ハッシュによるファイル存在チェック
pub async fn check_existence(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    let md5 = query.get("md5")
        .ok_or_else(|| AppError::Validation("md5 parameter required".to_string()))?;

    let file: Option<DriveFile> = state.surreal()
        .query("SELECT * FROM drive_file WHERE hash = $hash AND owner_id = $owner_id")
        .bind(("hash", md5.clone()))
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    if let Some(f) = file {
        Ok(Json(serde_json::json!({
            "exists": true,
            "file": FileResponse::from(&f)
        })))
    } else {
        Ok(Json(serde_json::json!({ "exists": false })))
    }
}

/// ハッシュ値によるファイル検索
pub async fn find_by_hash(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<FileResponse>>> {
    let hash = query.get("md5")
        .or_else(|| query.get("sha256"))
        .ok_or_else(|| AppError::Validation("md5 or sha256 parameter required".to_string()))?;

    let files: Vec<DriveFile> = state.surreal()
        .query("SELECT * FROM drive_file WHERE hash = $hash AND owner_id = $owner_id")
        .bind(("hash", hash.clone()))
        .bind(("owner_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    Ok(Json(files.iter().map(FileResponse::from).collect()))
}

/// 条件によるファイル検索
pub async fn find_files(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Query(query): Query<FindFilesQuery>,
) -> Result<Json<Vec<FileResponse>>> {
    let limit = query.limit.unwrap_or(10).min(100);
    let offset = query.offset.unwrap_or(0);

    let mut query_str = "SELECT * FROM drive_file WHERE owner_id = $owner_id".to_string();
    if query.name.is_some() { query_str.push_str(" AND name = $name"); }
    if query.folder_id.is_some() { query_str.push_str(" AND folder_id = $folder_id"); }
    if query.mime_type.is_some() { query_str.push_str(" AND mime_type = $mime_type"); }
    query_str.push_str(" ORDER BY created_at DESC LIMIT $limit START $offset");

    let mut surreal_query = state.surreal()
        .query(&query_str)
        .bind(("owner_id", auth_user.user_id.to_string()))
        .bind(("limit", limit))
        .bind(("offset", offset));

    if let Some(name) = query.name { surreal_query = surreal_query.bind(("name", name)); }
    if let Some(folder_id) = query.folder_id { surreal_query = surreal_query.bind(("folder_id", folder_id)); }
    if let Some(mime_type) = query.mime_type { surreal_query = surreal_query.bind(("mime_type", mime_type)); }

    let files: Vec<DriveFile> = surreal_query
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    Ok(Json(files.iter().map(FileResponse::from).collect()))
}

/// URLからファイルをアップロード
pub async fn upload_from_url(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Json(req): Json<UploadFromUrlRequest>,
) -> Result<Json<FileResponse>> {
    // Fetch file from URL
    let response = state.http_client()
        .get(&req.url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch URL: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Internal(format!("Remote server returned {}", response.status())));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .split(';')
        .next()
        .unwrap_or("application/octet-stream")
        .to_string();

    let data = response.bytes().await
        .map_err(|e| AppError::Internal(format!("Failed to read response body: {}", e)))?;

    let size = data.len() as i64;
    if size > 100 * 1024 * 1024 {
        return Err(AppError::Validation("File too large (max 100MB)".to_string()));
    }

    let hash = format!("{:x}", Sha256::digest(&data));

    // Deduplication check (skip if force=true)
    if !req.force.unwrap_or(false) {
        let existing: Option<DriveFile> = state.surreal()
            .query("SELECT * FROM drive_file WHERE hash = $hash AND owner_id = $owner_id")
            .bind(("hash", hash.clone()))
            .bind(("owner_id", auth_user.user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .take(0)
            .ok()
            .flatten();

        if let Some(file) = existing {
            return Ok(Json(FileResponse::from(&file)));
        }
    }

    // Determine filename
    let file_name = req.name.unwrap_or_else(|| {
        req.url.split('/').last().unwrap_or("file").to_string()
    });

    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
    let file_id = Ulid::new();
    let ext = std::path::Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let save_name = if ext.is_empty() {
        file_id.to_string()
    } else {
        format!("{}.{}", file_id, ext)
    };
    let file_path = std::path::Path::new(&upload_dir).join(&save_name);

    std::fs::create_dir_all(&upload_dir).map_err(|e| {
        AppError::Internal(format!("Failed to create upload directory: {}", e))
    })?;

    let mut f = std::fs::File::create(&file_path).map_err(|e| {
        AppError::Internal(format!("Failed to create file: {}", e))
    })?;
    f.write_all(&data).map_err(|e| {
        AppError::Internal(format!("Failed to write file: {}", e))
    })?;

    let (width, height) = if content_type.starts_with("image/") {
        get_image_dimensions(&data)
    } else {
        (None, None)
    };

    let path_str = file_path.to_string_lossy().to_string();
    let mut drive_file = DriveFile::new(file_name, content_type, size, auth_user.user_id, path_str, hash);
    drive_file.width = width;
    drive_file.height = height;

    let created: DriveFile = state.surreal()
        .create(("drive_file", drive_file.id.to_string()))
        .content(drive_file)
        .await
        .map_err(|e| {
            error!("Failed to create drive file record: {}", e);
            AppError::Database(e)
        })?;

    Ok(Json(FileResponse::from(&created)))
}
