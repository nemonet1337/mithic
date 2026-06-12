use crate::SurrealClient;
use mithic_core::models::actor::ActorId;
use mithic_core::models::file::{DriveFile, FileId, FileType};

fn map_row_to_file(val: serde_json::Value) -> Option<DriveFile> {
    let id_str = val.get("id")?.as_str()?;
    let id = id_str.parse::<FileId>().ok()?;
    let created_at_str = val.get("created_at")?.as_str()?;
    let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
        .ok()?
        .with_timezone(&chrono::Utc);

    let name = val.get("name")?.as_str()?.to_string();
    let mime_type = val.get("mime_type")?.as_str()?.to_string();
    let size = val.get("size")?.as_i64()?;

    let owner_id_str = val.get("owner_id")?.as_str()?;
    let owner_id = owner_id_str.parse::<ActorId>().ok()?;

    let hash = val
        .get("hash")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let url = val
        .get("url")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let thumbnail_url = val
        .get("thumbnail_url")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let file_type = if mime_type.starts_with("image/") {
        FileType::Image
    } else if mime_type.starts_with("video/") {
        FileType::Video
    } else if mime_type.starts_with("audio/") {
        FileType::Audio
    } else {
        FileType::Other
    };

    Some(DriveFile {
        id,
        created_at,
        updated_at: None,
        name,
        mime_type,
        file_type,
        size,
        owner_id,
        path: "".to_string(),
        thumbnail_path: None,
        url,
        thumbnail_url,
        width: None,
        height: None,
        duration: None,
        hash,
        is_public: true,
        folder_id: None,
        comment: None,
    })
}

pub async fn create_drive_file(client: &SurrealClient, file: &DriveFile) -> anyhow::Result<()> {
    let id_str = file.id.to_string();
    let owner_str = file.owner_id.to_string();

    client
        .query(
            "
            INSERT INTO drive_file {
                id: $id,
                user_id: type::record('user', $owner),
                name: $name,
                mime_type: $mime_type,
                size: $size,
                md5: $hash,
                url: $url,
                thumbnail_url: $thumbnail_url,
                created_at: $created_at
            };
            ",
        )
        .bind(("id", id_str))
        .bind(("owner", owner_str))
        .bind(("name", file.name.clone()))
        .bind(("mime_type", file.mime_type.clone()))
        .bind(("size", file.size))
        .bind(("hash", file.hash.clone()))
        .bind(("url", file.url.clone()))
        .bind(("thumbnail_url", file.thumbnail_url.clone()))
        .bind(("created_at", file.created_at))
        .await?;

    Ok(())
}

pub async fn get_drive_file(
    client: &SurrealClient,
    id: &FileId,
) -> anyhow::Result<Option<DriveFile>> {
    let id_str = id.to_string();

    let mut response = client
        .query(
            "
            SELECT 
                id,
                created_at,
                name,
                mime_type,
                size,
                user_id.id AS owner_id,
                md5 AS hash,
                url,
                thumbnail_url
            FROM drive_file
            WHERE id = type::record('drive_file', $id)
            LIMIT 1;
            ",
        )
        .bind(("id", id_str))
        .await?;

    let rows: Vec<serde_json::Value> = response.take(0)?;
    if let Some(row) = rows.into_iter().next() {
        Ok(map_row_to_file(row))
    } else {
        Ok(None)
    }
}

pub async fn get_user_drive_files(
    client: &SurrealClient,
    owner_id: &ActorId,
    limit: usize,
) -> anyhow::Result<Vec<DriveFile>> {
    let owner_str = owner_id.to_string();

    let mut response = client
        .query(
            "
            SELECT 
                id,
                created_at,
                name,
                mime_type,
                size,
                user_id.id AS owner_id,
                md5 AS hash,
                url,
                thumbnail_url
            FROM drive_file
            WHERE user_id = type::record('user', $owner)
            ORDER BY created_at DESC
            LIMIT $limit;
            ",
        )
        .bind(("owner", owner_str))
        .bind(("limit", limit))
        .await?;

    let rows: Vec<serde_json::Value> = response.take(0)?;
    let mut files = Vec::new();
    for row in rows {
        if let Some(file) = map_row_to_file(row) {
            files.push(file);
        }
    }
    Ok(files)
}

pub async fn delete_drive_file(client: &SurrealClient, id: &FileId) -> anyhow::Result<()> {
    let id_str = id.to_string();

    client
        .query(
            "
            DELETE drive_file WHERE id = type::record('drive_file', $id);
            ",
        )
        .bind(("id", id_str))
        .await?;

    Ok(())
}
