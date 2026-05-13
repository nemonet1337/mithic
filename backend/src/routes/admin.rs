//! Admin API endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    services::federation::{QueueJob, QueueStats},
    state::{AppState, AuthUser},
};

fn require_admin(auth_user: &AuthUser) -> Result<()> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct QueueJobsQuery {
    pub limit: Option<usize>,
    pub host: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsersQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminDriveFilesQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserFilesRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminChangePasswordRequest {
    pub user_id: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMetaRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub banner_url: Option<String>,
    pub icon_url: Option<String>,
    pub maintainer_name: Option<String>,
    pub maintainer_email: Option<String>,
    pub langs: Option<Vec<String>>,
    pub tos_url: Option<String>,
    pub repository_url: Option<String>,
    pub feedback_url: Option<String>,
    pub disable_registration: Option<bool>,
    pub disable_local_timeline: Option<bool>,
    pub disable_global_timeline: Option<bool>,
    pub enable_email: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRemoteUserRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInstanceRequest {
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct AddEmojiRequest {
    pub name: String,
    pub file_id: String,
    pub category: Option<String>,
    pub aliases: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmojiRequest {
    pub name: Option<String>,
    pub category: Option<String>,
    pub aliases: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ShowDriveFileRequest {
    pub file_id: Option<String>,
    pub url: Option<String>,
}

// ─── Queue ─────────────────────────────────────────────────────────────────

/// Get queue statistics (admin only)
pub async fn get_queue_stats(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<QueueStats>> {
    require_admin(&auth_user)?;
    let stats = state.federation_service().get_queue_stats().await
        .map_err(|e| AppError::Internal(format!("Failed to get queue stats: {}", e)))?;
    Ok(Json(stats))
}

/// Get queue jobs (admin only)
pub async fn get_queue_jobs(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<QueueJobsQuery>,
) -> Result<Json<Vec<QueueJob>>> {
    require_admin(&auth_user)?;
    let limit = query.limit.unwrap_or(50);
    let jobs = state.federation_service().get_queue_jobs(limit).await
        .map_err(|e| AppError::Internal(format!("Failed to get queue jobs: {}", e)))?;
    Ok(Json(jobs))
}

/// Get deliver-delayed queue jobs grouped by host (admin only)
pub async fn get_deliver_delayed(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<QueueJobsQuery>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    let pattern = "federation:queue:deliver:*";
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(pattern)
        .query_async(&mut state.dragonfly().clone())
        .await
        .unwrap_or_default();

    let mut result: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for key in &keys {
        let host = key.trim_start_matches("federation:queue:deliver:").to_string();
        let len: i64 = redis::cmd("LLEN")
            .arg(key)
            .query_async(&mut state.dragonfly().clone())
            .await
            .unwrap_or(0);
        result.insert(host, len);
    }

    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// Get inbox-delayed queue jobs grouped by host (admin only)
pub async fn get_inbox_delayed(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<QueueJobsQuery>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    let pattern = "federation:queue:inbox:*";
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(pattern)
        .query_async(&mut state.dragonfly().clone())
        .await
        .unwrap_or_default();

    let mut result: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for key in &keys {
        let host = key.trim_start_matches("federation:queue:inbox:").to_string();
        let len: i64 = redis::cmd("LLEN")
            .arg(key)
            .query_async(&mut state.dragonfly().clone())
            .await
            .unwrap_or(0);
        result.insert(host, len);
    }

    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// Clear federation queue (admin only)
pub async fn clear_queue(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;
    redis::cmd("DEL")
        .arg("federation:queue")
        .query_async::<_, ()>(&mut state.dragonfly().clone())
        .await
        .ok();
    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── Users ──────────────────────────────────────────────────────────────────

/// Show users (admin only)
pub async fn show_users(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<UsersQuery>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut result = state.surreal()
        .query("SELECT * FROM user LIMIT $limit START $offset")
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await
        .map_err(|e| AppError::Database(e))?;

    let users: Vec<serde_json::Value> = result.take(0).map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    Ok(Json(serde_json::json!({ "users": users, "count": users.len() })))
}

/// Create user account (admin only)
pub async fn create_user_account(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let username_lower = req.username.to_lowercase();

    // Check if username is already taken
    let existing: Option<serde_json::Value> = state.surreal()
        .query("SELECT id FROM user WHERE username_lower = $username")
        .bind(("username", username_lower.clone()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .map_err(|e| AppError::Database(e))?;

    if existing.is_some() {
        return Err(AppError::Conflict("Username already taken".to_string()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();

    let user_id = Ulid::new();
    state.surreal()
        .query("CREATE user:$id SET username = $username, username_lower = $username_lower, password_hash = $password_hash, email = $email, is_admin = false, is_suspended = false, is_locked = false, is_bot = false, followers_count = 0, following_count = 0, notes_count = 0, created_at = time::now()")
        .bind(("id", user_id.to_string()))
        .bind(("username", req.username))
        .bind(("username_lower", username_lower))
        .bind(("password_hash", password_hash))
        .bind(("email", req.email))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true, "id": user_id.to_string() })))
}

/// Delete user account (admin only)
pub async fn delete_user_account(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;
    let actor_id = user_id.parse::<Ulid>()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    // Track the username as used
    let username: Option<String> = state.surreal()
        .query("SELECT username_lower FROM user WHERE id = $id")
        .bind(("id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten()
        .and_then(|v: serde_json::Value| v.get("username_lower").and_then(|s| s.as_str()).map(|s| s.to_string()));

    if let Some(uname) = username {
        let used_id = Ulid::new();
        state.surreal()
            .query("CREATE used_username:$id SET username = $username, created_at = time::now()")
            .bind(("id", used_id.to_string()))
            .bind(("username", uname))
            .await
            .ok();
    }

    state.surreal()
        .query("DELETE FROM user WHERE id = $id")
        .bind(("id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Suspend user (admin only)
pub async fn suspend_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;
    let actor_id = user_id.parse::<Ulid>()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    state.surreal()
        .query("UPDATE user SET is_suspended = true WHERE id = $id")
        .bind(("id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Unsuspend user (admin only)
pub async fn unsuspend_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;
    let actor_id = user_id.parse::<Ulid>()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    state.surreal()
        .query("UPDATE user SET is_suspended = false WHERE id = $id")
        .bind(("id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Admin change password (admin only)
pub async fn admin_change_password(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<AdminChangePasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let actor_id = req.user_id.parse::<Ulid>()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();

    state.surreal()
        .query("UPDATE user SET password_hash = $hash WHERE id = $id")
        .bind(("id", actor_id.to_string()))
        .bind(("hash", password_hash))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Update remote user info (admin only)
pub async fn update_remote_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateRemoteUserRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;
    let actor_id = req.user_id.parse::<Ulid>()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    // Re-fetch actor info from remote
    let actor: Option<serde_json::Value> = state.surreal()
        .query("SELECT uri FROM user WHERE id = $id AND host IS NOT NULL")
        .bind(("id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .map_err(|e| AppError::Database(e))?;

    let actor = actor.ok_or_else(|| AppError::NotFound("Remote user not found".to_string()))?;
    let uri = actor.get("uri").and_then(|v| v.as_str()).unwrap_or_default().to_string();

    // Fetch updated actor from remote
    if !uri.is_empty() {
        let response = state.http_client()
            .get(&uri)
            .header("Accept", "application/activity+json")
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch remote actor: {}", e)))?;

        if response.status().is_success() {
            let remote: serde_json::Value = response.json().await
                .map_err(|e| AppError::Internal(format!("Failed to parse remote actor: {}", e)))?;

            let name = remote.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
            let summary = remote.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());

            state.surreal()
                .query("UPDATE user SET name = $name, bio = $bio, updated_at = time::now() WHERE id = $id")
                .bind(("id", actor_id.to_string()))
                .bind(("name", name))
                .bind(("bio", summary))
                .await
                .map_err(|e| AppError::Database(e))?;
        }
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── Meta ───────────────────────────────────────────────────────────────────

/// Update server metadata (admin only)
pub async fn update_meta(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateMetaRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    let mut updates = Vec::new();
    if let Some(name) = &req.name { updates.push(format!("name = '{}'", name.replace('\'', "''"))); }
    if let Some(desc) = &req.description { updates.push(format!("description = '{}'", desc.replace('\'', "''"))); }
    if let Some(url) = &req.banner_url { updates.push(format!("banner_url = '{}'", url.replace('\'', "''"))); }
    if let Some(url) = &req.icon_url { updates.push(format!("icon_url = '{}'", url.replace('\'', "''"))); }
    if let Some(v) = req.disable_registration { updates.push(format!("disable_registration = {}", v)); }
    if let Some(v) = req.disable_local_timeline { updates.push(format!("disable_local_timeline = {}", v)); }
    if let Some(v) = req.disable_global_timeline { updates.push(format!("disable_global_timeline = {}", v)); }

    if !updates.is_empty() {
        let query = format!("UPDATE instance_config SET {}", updates.join(", "));
        state.surreal().query(&query).await.map_err(|e| AppError::Database(e))?;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── Database ───────────────────────────────────────────────────────────────

/// Get table statistics (admin only)
pub async fn get_table_stats(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    let tables = ["user", "note", "follow", "reaction", "drive_file", "notification", "hashtag", "relay"];
    let mut stats = serde_json::Map::new();

    for table in &tables {
        let count: Option<serde_json::Value> = state.surreal()
            .query(format!("SELECT count() FROM {} GROUP ALL", table))
            .await
            .map_err(|e| AppError::Database(e))?
            .take(0)
            .ok()
            .flatten();

        let n = count.as_ref()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        stats.insert(table.to_string(), serde_json::Value::Number(n.into()));
    }

    Ok(Json(serde_json::Value::Object(stats)))
}

/// Vacuum database (admin only)
pub async fn vacuum(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    // SurrealDB doesn't have VACUUM, but we clean up orphaned records
    state.surreal()
        .query("DELETE FROM notification WHERE recipient_id NOT IN (SELECT id FROM user)")
        .await
        .map_err(|e| AppError::Database(e))?;

    state.surreal()
        .query("DELETE FROM follow WHERE in NOT IN (SELECT id FROM user) OR out NOT IN (SELECT id FROM user)")
        .await
        .ok();

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Resync chart data (admin only)
pub async fn resync_chart(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    // Rebuild chart entries from raw data
    let user_count: Option<serde_json::Value> = state.surreal()
        .query("SELECT count() FROM user GROUP ALL")
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    let note_count: Option<serde_json::Value> = state.surreal()
        .query("SELECT count() FROM note GROUP ALL")
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    Ok(Json(serde_json::json!({
        "success": true,
        "users": user_count,
        "notes": note_count,
    })))
}

// ─── Emoji ──────────────────────────────────────────────────────────────────

/// Add custom emoji (admin only)
pub async fn add_emoji(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<AddEmojiRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    let emoji_id = Ulid::new();
    state.surreal()
        .query("CREATE emoji:$id SET name = $name, file_id = $file_id, category = $category, aliases = $aliases, is_self_hosted = true, created_at = time::now()")
        .bind(("id", emoji_id.to_string()))
        .bind(("name", &req.name))
        .bind(("file_id", &req.file_id))
        .bind(("category", req.category.as_deref().unwrap_or("")))
        .bind(("aliases", req.aliases.clone().unwrap_or_default()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true, "id": emoji_id.to_string() })))
}

/// Update custom emoji (admin only)
pub async fn update_emoji(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(emoji_id): Path<String>,
    Json(req): Json<UpdateEmojiRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    let id = emoji_id.parse::<Ulid>()
        .map_err(|_| AppError::BadRequest("Invalid emoji ID".to_string()))?;

    let mut updates = Vec::new();
    if let Some(name) = &req.name { updates.push(format!("name = '{}'", name.replace('\'', "''"))); }
    if let Some(cat) = &req.category { updates.push(format!("category = '{}'", cat.replace('\'', "''"))); }

    if !updates.is_empty() {
        let query = format!("UPDATE emoji:{} SET {}", id, updates.join(", "));
        state.surreal().query(&query).await.map_err(|e| AppError::Database(e))?;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Remove custom emoji (admin only)
pub async fn remove_emoji(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(emoji_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    let id = emoji_id.parse::<Ulid>()
        .map_err(|_| AppError::BadRequest("Invalid emoji ID".to_string()))?;

    state.surreal()
        .query("DELETE emoji WHERE id = $id")
        .bind(("id", id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── Federation ─────────────────────────────────────────────────────────────

/// Update remote instance metadata (admin only)
pub async fn federation_update_instance(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateInstanceRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    // Fetch updated nodeinfo from remote
    let nodeinfo_url = format!("https://{}/.well-known/nodeinfo", req.host);
    let response = state.http_client()
        .get(&nodeinfo_url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch nodeinfo: {}", e)))?;

    if response.status().is_success() {
        let nodeinfo: serde_json::Value = response.json().await.unwrap_or_default();
        let software = nodeinfo
            .get("software")
            .and_then(|s| s.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        state.surreal()
            .query("UPDATE instance SET software = $software, updated_at = time::now() WHERE host = $host")
            .bind(("host", req.host))
            .bind(("software", software))
            .await
            .map_err(|e| AppError::Database(e))?;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Remove all following relationships to a remote instance (admin only)
pub async fn federation_remove_all_following(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateInstanceRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    state.surreal()
        .query("DELETE follow WHERE out IN (SELECT id FROM user WHERE host = $host)")
        .bind(("host", req.host))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Delete all files from a remote instance (admin only)
pub async fn federation_delete_all_files(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateInstanceRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    state.surreal()
        .query("DELETE drive_file WHERE owner_id IN (SELECT id FROM user WHERE host = $host)")
        .bind(("host", req.host))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── Drive ──────────────────────────────────────────────────────────────────

/// Get all drive files (admin only)
pub async fn get_all_drive_files(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<AdminDriveFilesQuery>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let (query_str, mut surreal_query) = if let Some(user_id) = &query.user_id {
        let q = "SELECT * FROM drive_file WHERE owner_id = $user_id LIMIT $limit START $offset";
        let sq = state.surreal()
            .query(q)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .bind(("user_id", user_id.clone()));
        (q, sq)
    } else {
        let q = "SELECT * FROM drive_file LIMIT $limit START $offset";
        let sq = state.surreal()
            .query(q)
            .bind(("limit", limit))
            .bind(("offset", offset));
        (q, sq)
    };

    let files: Vec<serde_json::Value> = surreal_query.await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    Ok(Json(serde_json::json!({ "files": files, "count": files.len() })))
}

/// Delete all files of a user (admin only)
pub async fn delete_all_files_of_a_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<DeleteUserFilesRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;
    let actor_id = req.user_id.parse::<Ulid>()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    state.surreal()
        .query("DELETE FROM drive_file WHERE owner_id = $user_id")
        .bind(("user_id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Clean remote cache files (admin only)
pub async fn drive_clean_remote_files(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    let deleted: Vec<serde_json::Value> = state.surreal()
        .query("SELECT id, path FROM drive_file WHERE owner_id IN (SELECT id FROM user WHERE host IS NOT NULL)")
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    let count = deleted.len();

    // Delete file records
    state.surreal()
        .query("DELETE FROM drive_file WHERE owner_id IN (SELECT id FROM user WHERE host IS NOT NULL)")
        .await
        .map_err(|e| AppError::Database(e))?;

    // Delete physical files
    for file in &deleted {
        if let Some(path) = file.get("path").and_then(|v| v.as_str()) {
            std::fs::remove_file(path).ok();
        }
    }

    Ok(Json(serde_json::json!({ "success": true, "deleted": count })))
}

/// Drive database cleanup (admin only)
pub async fn drive_cleanup(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    // Remove drive files with no owner
    state.surreal()
        .query("DELETE FROM drive_file WHERE owner_id NOT IN (SELECT id FROM user)")
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Show a specific drive file (admin only)
pub async fn drive_show_file(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ShowDriveFileRequest>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&auth_user)?;

    let file: Option<serde_json::Value> = if let Some(file_id) = &query.file_id {
        let id = file_id.parse::<Ulid>()
            .map_err(|_| AppError::BadRequest("Invalid file ID".to_string()))?;
        state.surreal()
            .query("SELECT * FROM drive_file WHERE id = $id")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .take(0)
            .ok()
            .flatten()
    } else if let Some(url) = &query.url {
        state.surreal()
            .query("SELECT * FROM drive_file WHERE url = $url")
            .bind(("url", url.clone()))
            .await
            .map_err(|e| AppError::Database(e))?
            .take(0)
            .ok()
            .flatten()
    } else {
        return Err(AppError::BadRequest("file_id or url is required".to_string()));
    };

    let file = file.ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    Ok(Json(file))
}
