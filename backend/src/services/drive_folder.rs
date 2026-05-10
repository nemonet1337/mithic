use chrono::{DateTime, Utc};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::actor::ActorId,
    state::AppState,
};

/// ドライブフォルダID
pub type FolderId = String;

/// ドライブフォルダサービス
pub struct DriveFolderService;

impl DriveFolderService {
    /// フォルダを作成
    pub async fn create_folder(
        state: &AppState,
        user_id: ActorId,
        name: String,
        parent_id: Option<String>,
    ) -> Result<DriveFolder> {
        // 親フォルダの所有権チェック
        if let Some(parent_id) = &parent_id {
            let parent = Self::get_folder(state, parent_id.clone(), user_id).await?;
            if parent.is_none() {
                return Err(AppError::Validation("Parent folder not found".to_string()));
            }
        }

        let id = FolderId::from(Ulid::new().to_string());
        let created_at = Utc::now();

        let query = r#"
            CREATE drive_folder:$id SET
                id = $id,
                created_at = $created_at,
                updated_at = $created_at,
                name = $name,
                owner_id = $owner_id,
                parent_id = $parent_id
        "#;

        let mut result = state
            .surreal()
            .query(query)
            .bind(("id", id.clone()))
            .bind(("created_at", created_at))
            .bind(("name", name))
            .bind(("owner_id", user_id.to_string()))
            .bind(("parent_id", parent_id))
            .await
            .map_err(|e| AppError::Database(e))?;

        let folder: DriveFolder = result.take(0).map_err(|e| AppError::Database(e))?;

        Ok(folder)
    }

    /// フォルダを更新
    pub async fn update_folder(
        state: &AppState,
        folder_id: String,
        user_id: ActorId,
        name: Option<String>,
        parent_id: Option<Option<String>>,
    ) -> Result<DriveFolder> {
        // フォルダの所有権チェック
        let folder = Self::get_folder(state, folder_id.clone(), user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Folder not found".to_string()))?;

        // 循環参照チェック
        if let Some(Some(new_parent_id)) = parent_id {
            if new_parent_id == folder_id {
                return Err(AppError::Validation("Cannot set parent to self".to_string()));
            }

            if let Some(current_parent_id) = &folder.parent_id {
                if Self::has_circular_reference(state, &new_parent_id, current_parent_id).await? {
                    return Err(AppError::Validation("Circular reference detected".to_string()));
                }
            }

            // 親フォルダの所有権チェック
            let parent = Self::get_folder(state, new_parent_id.clone(), user_id).await?;
            if parent.is_none() {
                return Err(AppError::Validation("Parent folder not found".to_string()));
            }
        }

        let mut updates = vec![];
        if let Some(n) = name {
            updates.push(format!("name = '{}", n.replace("'", "''")));
        }
        if let Some(p) = parent_id {
            updates.push(format!("parent_id = {}", match p {
                Some(id) => format!("'{}'", id),
                None => "NONE".to_string(),
            }));
        }
        updates.push("updated_at = time::now()".to_string());

        if updates.is_empty() {
            return Ok(folder);
        }

        let query = format!(
            "UPDATE drive_folder:{} SET {}",
            folder_id,
            updates.join(", ")
        );

        state
            .surreal()
            .query(&query)
            .await
            .map_err(|e| AppError::Database(e))?;

        Self::get_folder(state, folder_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Folder not found".to_string()))
    }

    /// フォルダを取得
    pub async fn get_folder(
        state: &AppState,
        folder_id: String,
        user_id: ActorId,
    ) -> Result<Option<DriveFolder>> {
        let query = r#"
            SELECT * FROM drive_folder
            WHERE id = $folder_id AND owner_id = $user_id
        "#;

        let mut result = state
            .surreal()
            .query(query)
            .bind(("folder_id", folder_id))
            .bind(("user_id", user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let folder: Option<DriveFolder> = result.take(0).map_err(|e| AppError::Database(e))?;

        Ok(folder)
    }

    /// フォルダを削除
    pub async fn delete_folder(
        state: &AppState,
        folder_id: String,
        user_id: ActorId,
    ) -> Result<()> {
        // フォルダの所有権チェック
        let folder = Self::get_folder(state, folder_id.clone(), user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Folder not found".to_string()))?;

        // 子フォルダを削除（再帰的）
        Self::delete_subfolders(state, &folder_id, user_id).await?;

        // フォルダを削除
        let query = "DELETE drive_folder WHERE id = $folder_id";

        state
            .surreal()
            .query(query)
            .bind(("folder_id", folder_id))
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    /// フォルダ一覧を取得
    pub async fn list_folders(
        state: &AppState,
        user_id: ActorId,
        parent_id: Option<String>,
    ) -> Result<Vec<DriveFolder>> {
        let query = if let Some(pid) = parent_id {
            r#"
                SELECT * FROM drive_folder
                WHERE owner_id = $user_id AND parent_id = $parent_id
                ORDER BY created_at DESC
            "#
        } else {
            r#"
                SELECT * FROM drive_folder
                WHERE owner_id = $user_id AND parent_id = NONE
                ORDER BY created_at DESC
            "#
        };

        let mut result = state
            .surreal()
            .query(query)
            .bind(("user_id", user_id.to_string()))
            .bind(("parent_id", parent_id))
            .await
            .map_err(|e| AppError::Database(e))?;

        let folders: Vec<DriveFolder> = result.take(0).map_err(|e| AppError::Database(e))?;

        Ok(folders)
    }

    /// 循環参照チェック
    async fn has_circular_reference(
        state: &AppState,
        new_parent_id: &str,
        current_parent_id: &str,
    ) -> Result<bool> {
        if new_parent_id == current_parent_id {
            return Ok(true);
        }

        let query = r#"
            SELECT parent_id FROM drive_folder WHERE id = $folder_id
        "#;

        let mut result = state
            .surreal()
            .query(query)
            .bind(("folder_id", new_parent_id))
            .await
            .map_err(|e| AppError::Database(e))?;

        let folder: Option<DriveFolder> = result.take(0).map_err(|e| AppError::Database(e))?;

        if let Some(folder) = folder {
            if let Some(parent_id) = folder.parent_id {
                return Self::has_circular_reference(state, &parent_id, current_parent_id).await;
            }
        }

        Ok(false)
    }

    /// 子フォルダを再帰的に削除
    async fn delete_subfolders(
        state: &AppState,
        parent_id: &str,
        user_id: ActorId,
    ) -> Result<()> {
        let query = r#"
            SELECT id FROM drive_folder
            WHERE parent_id = $parent_id AND owner_id = $user_id
        "#;

        let mut result = state
            .surreal()
            .query(query)
            .bind(("parent_id", parent_id))
            .bind(("user_id", user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let folders: Vec<DriveFolder> = result.take(0).map_err(|e| AppError::Database(e))?;

        for folder in folders {
            Self::delete_subfolders(state, &folder.id, user_id).await?;
            Self::delete_folder(state, folder.id, user_id).await?;
        }

        Ok(())
    }
}

/// ドライブフォルダ
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFolder {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub name: String,
    pub owner_id: ActorId,
    pub parent_id: Option<String>,
}
