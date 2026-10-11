use crate::db::Db;
use crate::models::actor::ActorId;
use crate::models::file::{DriveFile, FileId, FileType};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// drive_files の取得カラム。`map_row_to_file` はこの行 1 件を `DriveFile` に変換する。
pub(crate) const DRIVE_FILE_COLUMNS: &str = "
    id, created_at, owner_id, name, mime_type, size, hash, url, thumbnail_url
";

#[derive(Debug, FromRow)]
pub(crate) struct DriveFileRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub owner_id: String,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub hash: String,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
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

/// drive_files 1 行を `DriveFile` に変換する (routes/v1/drive.rs からも再利用)
pub(crate) fn map_row_to_file(row: DriveFileRow) -> Option<DriveFile> {
    let id = row.id.parse::<FileId>().ok()?;
    let owner_id = row.owner_id.parse::<ActorId>().ok()?;

    Some(DriveFile {
        id,
        created_at: row.created_at,
        updated_at: None,
        name: row.name,
        file_type: detect_file_type(&row.mime_type),
        mime_type: row.mime_type,
        size: row.size,
        owner_id,
        path: String::new(),
        thumbnail_path: None,
        url: row.url,
        thumbnail_url: row.thumbnail_url,
        width: None,
        height: None,
        duration: None,
        hash: row.hash,
        is_public: true,
        comment: None,
    })
}

fn rows_to_files(rows: Vec<DriveFileRow>) -> Vec<DriveFile> {
    rows.into_iter().filter_map(map_row_to_file).collect()
}

pub async fn create_drive_file(db: &Db, file: &DriveFile) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO drive_files (
            id, created_at, owner_id, name, mime_type, size, hash, url, thumbnail_url
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(file.id.to_string())
    .bind(file.created_at)
    .bind(file.owner_id.to_string())
    .bind(&file.name)
    .bind(&file.mime_type)
    .bind(file.size)
    .bind(&file.hash)
    .bind(&file.url)
    .bind(&file.thumbnail_url)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_drive_file(db: &Db, id: &FileId) -> anyhow::Result<Option<DriveFile>> {
    let row = sqlx::query_as::<_, DriveFileRow>(&format!(
        "SELECT {DRIVE_FILE_COLUMNS} FROM drive_files WHERE id = $1"
    ))
    .bind(id.to_string())
    .fetch_optional(db)
    .await?;

    Ok(row.and_then(map_row_to_file))
}

pub async fn get_drive_files_by_ids(db: &Db, ids: &[String]) -> anyhow::Result<Vec<DriveFile>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query_as::<_, DriveFileRow>(&format!(
        "SELECT {DRIVE_FILE_COLUMNS} FROM drive_files WHERE id = ANY($1)"
    ))
    .bind(ids)
    .fetch_all(db)
    .await?;

    Ok(rows_to_files(rows))
}

pub async fn get_user_drive_files(
    db: &Db,
    owner_id: &ActorId,
    limit: usize,
) -> anyhow::Result<Vec<DriveFile>> {
    let rows = sqlx::query_as::<_, DriveFileRow>(&format!(
        "SELECT {DRIVE_FILE_COLUMNS} FROM drive_files
         WHERE owner_id = $1
         ORDER BY created_at DESC
         LIMIT $2"
    ))
    .bind(owner_id.to_string())
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    Ok(rows_to_files(rows))
}

pub async fn delete_drive_file(db: &Db, id: &FileId) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM drive_files WHERE id = $1")
        .bind(id.to_string())
        .execute(db)
        .await?;
    Ok(())
}

pub async fn get_drive_file_by_hash(db: &Db, hash: &str) -> anyhow::Result<Option<DriveFile>> {
    let row = sqlx::query_as::<_, DriveFileRow>(&format!(
        "SELECT {DRIVE_FILE_COLUMNS} FROM drive_files WHERE hash = $1 LIMIT 1"
    ))
    .bind(hash)
    .fetch_optional(db)
    .await?;

    Ok(row.and_then(map_row_to_file))
}
