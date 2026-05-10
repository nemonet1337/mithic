use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::actor::ActorId,
    services::drive_folder::{DriveFolder, DriveFolderService},
    state::{AppState, AuthUser},
};

#[derive(Debug, Serialize)]
pub struct DriveFolderResponse {
    pub id: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub name: String,
    pub owner_id: String,
    pub parent_id: Option<String>,
}

impl From<DriveFolder> for DriveFolderResponse {
    fn from(folder: DriveFolder) -> Self {
        Self {
            id: folder.id,
            created_at: folder.created_at.to_rfc3339(),
            updated_at: folder.updated_at.map(|t| t.to_rfc3339()),
            name: folder.name,
            owner_id: folder.owner_id.to_string(),
            parent_id: folder.parent_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFolderRequest {
    pub name: Option<String>,
    pub parent_id: Option<Option<String>>,
}

/// フォルダを作成
pub async fn create_folder(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<DriveFolderResponse>> {
    let name = req.name.unwrap_or_else(|| "Untitled".to_string());
    let parent_id = req.parent_id;

    let folder = DriveFolderService::create_folder(&state, auth_user.user_id, name, parent_id).await?;

    Ok(Json(DriveFolderResponse::from(folder)))
}

/// フォルダを更新
pub async fn update_folder(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
    Json(req): Json<UpdateFolderRequest>,
) -> Result<Json<DriveFolderResponse>> {
    let folder = DriveFolderService::update_folder(
        &state,
        id,
        auth_user.user_id,
        req.name,
        req.parent_id,
    )
    .await?;

    Ok(Json(DriveFolderResponse::from(folder)))
}

/// フォルダを取得
pub async fn get_folder(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<DriveFolderResponse>> {
    let folder = DriveFolderService::get_folder(&state, id, auth_user.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Folder not found".to_string()))?;

    Ok(Json(DriveFolderResponse::from(folder)))
}

/// フォルダを削除
pub async fn delete_folder(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    DriveFolderService::delete_folder(&state, id, auth_user.user_id).await?;

    Ok(Json(serde_json::json!({})))
}

/// フォルダ一覧を取得
pub async fn list_folders(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<Vec<DriveFolderResponse>>> {
    let folders = DriveFolderService::list_folders(&state, auth_user.user_id, None).await?;

    Ok(Json(folders.into_iter().map(DriveFolderResponse::from).collect()))
}
