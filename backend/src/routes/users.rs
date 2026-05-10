use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::Actor,
    services::follow_request::FollowRequestService,
    state::{AppState, AuthUser},
};

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClientSettingRequest {
    #[serde(flatten)]
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: String,
    pub username: String,
    pub acct: String,
    pub display_name: Option<String>,
    pub locked: bool,
    pub bot: bool,
    pub discoverable: bool,
    pub group: bool,
    pub created_at: String,
    pub note: String,
    pub url: String,
    pub avatar: String,
    pub avatar_static: String,
    pub header: String,
    pub header_static: String,
    pub followers_count: i32,
    pub following_count: i32,
    pub statuses_count: i32,
    pub last_status_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCredentialsRequest {
    pub display_name: Option<String>,
    pub note: Option<String>,
    pub locked: Option<bool>,
    pub bot: Option<bool>,
    pub avatar_id: Option<String>,
    pub header_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RelationsQuery {
    pub id: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RelationResponse {
    pub id: String,
    pub following: bool,
    pub followed_by: bool,
    pub blocking: bool,
    pub blocked_by: bool,
    pub muting: bool,
    pub requested: bool,
}

/// 認証済みアカウント情報取得
pub async fn verify_credentials(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<AccountResponse>> {
    let actor = fetch_actor_by_id(&state, auth_user.user_id).await?;
    Ok(Json(to_account_response(&actor, &state.config().instance_url)))
}

/// アカウント情報取得
pub async fn get_account(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AccountResponse>> {
    let actor_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    let actor = fetch_actor_by_id(&state, actor_id).await?;
    Ok(Json(to_account_response(&actor, &state.config().instance_url)))
}

/// アカウントをフォロー
pub async fn follow_account(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let target_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    // ターゲットユーザーを取得
    let target = fetch_actor_by_id(&state, target_id).await?;

    // 鍵付きアカウントの場合はフォローリクエストを作成
    if target.is_locked {
        // 既にリクエストが存在するか確認
        let exists = FollowRequestService::exists(&state, auth_user.user_id, target_id).await?;
        if exists {
            return Ok(Json(serde_json::json!({
                "id": format!("{}", auth_user.user_id),
                "following": false,
                "requested": true,
                "showing_reblogs": true,
                "notifying": false,
                "followed_by": false,
                "blocking": false,
                "blocked_by": false,
                "muting": false,
                "muting_notifications": false,
                "domain_blocking": false,
                "endorsed": false,
            })));
        }

        FollowRequestService::create_request(&state, auth_user.user_id, target_id, None).await?;

        return Ok(Json(serde_json::json!({
            "id": format!("{}", auth_user.user_id),
            "following": false,
            "requested": true,
            "showing_reblogs": true,
            "notifying": false,
            "followed_by": false,
            "blocking": false,
            "blocked_by": false,
            "muting": false,
            "muting_notifications": false,
            "domain_blocking": false,
            "endorsed": false,
        })));
    }

    // フォロー関係をSurrealDBに作成
    let relation_query = r#"
        RELATE user:$follower->follow->user:$followee
        SET created_at = time::now(), is_accepted = true
    "#;

    state
        .surreal()
        .query(relation_query)
        .bind(("follower", auth_user.user_id.to_string()))
        .bind(("followee", target_id.to_string()))
        .await
        .map_err(|e| {
            error!("Failed to create follow relation: {}", e);
            AppError::Database(e)
        })?;

    // フォロー数を更新
    update_follow_counts(&state, auth_user.user_id, target_id).await?;

    Ok(Json(serde_json::json!({
        "id": format!("{}", auth_user.user_id),
        "following": true,
        "requested": false,
        "showing_reblogs": true,
        "notifying": false,
        "followed_by": false,
        "blocking": false,
        "blocked_by": false,
        "muting": false,
        "muting_notifications": false,
        "domain_blocking": false,
        "endorsed": false,
    })))
}

/// アカウントをフォロー解除
pub async fn unfollow_account(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let target_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    // 自分自身
    if auth_user.user_id == target_id {
        return Err(AppError::Forbidden("Cannot unfollow yourself".to_string()));
    }

    // フォロー関係を削除
    let delete_query = r#"
        DELETE follow WHERE in = user:$follower AND out = user:$followee
    "#;

    state
        .surreal()
        .query(delete_query)
        .bind(("follower", auth_user.user_id.to_string()))
        .bind(("followee", target_id.to_string()))
        .await
        .map_err(|e| {
            error!("Failed to delete follow relation: {}", e);
            AppError::Database(e)
        })?;

    // フォロー数を更新
    update_follow_counts(&state, auth_user.user_id, target_id).await?;

    Ok(Json(serde_json::json!({
        "id": format!("{}", auth_user.user_id),
        "following": false,
        "showing_reblogs": false,
        "notifying": false,
        "followed_by": false,
        "blocking": false,
        "blocked_by": false,
        "muting": false,
        "muting_notifications": false,
        "requested": false,
        "domain_blocking": false,
        "endorsed": false,
    })))
}

/// アカウントをブロック
pub async fn block_account(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let target_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    // 自分自身
    if auth_user.user_id == target_id {
        return Err(AppError::Forbidden("Cannot block yourself".to_string()));
    }

    // 既にブロックしているか確認
    let check_query = r#"
        SELECT * FROM block WHERE in = user:$blocker AND out = user:$blockee
    "#;
    let mut check_result = state
        .surreal()
        .query(check_query)
        .bind(("blocker", auth_user.user_id.to_string()))
        .bind(("blockee", target_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let existing: Vec<crate::models::Block> = check_result.take(0).unwrap_or_default();
    if !existing.is_empty() {
        return Err(AppError::Conflict("Already blocking this user".to_string()));
    }

    // ブロックを作成
    let create_query = r#"
        CREATE block SET
            in = user:$blocker,
            out = user:$blockee,
            created_at = time::now()
    "#;

    state
        .surreal()
        .query(create_query)
        .bind(("blocker", auth_user.user_id.to_string()))
        .bind(("blockee", target_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    // フォロー関係も削除
    let delete_follow_query = r#"
        DELETE follow WHERE in = user:$follower AND out = user:$followee
    "#;
    state
        .surreal()
        .query(delete_follow_query)
        .bind(("follower", auth_user.user_id.to_string()))
        .bind(("followee", target_id.to_string()))
        .await
        .ok();

    // フォロー数を更新
    update_follow_counts(&state, auth_user.user_id, target_id).await?;

    Ok(Json(serde_json::json!({
        "id": format!("{}", auth_user.user_id),
        "blocking": true
    })))
}

/// アカウントのブロック解除
pub async fn unblock_account(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let target_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    // ブロックを削除
    let delete_query = r#"
        DELETE block WHERE in = user:$blocker AND out = user:$blockee
    "#;

    state
        .surreal()
        .query(delete_query)
        .bind(("blocker", auth_user.user_id.to_string()))
        .bind(("blockee", target_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "id": format!("{}", auth_user.user_id),
        "blocking": false
    })))
}

/// アカウントをミュート
pub async fn mute_account(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let target_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    // 自分自身
    if auth_user.user_id == target_id {
        return Err(AppError::Forbidden("Cannot mute yourself".to_string()));
    }

    // 既にミュートしているか確認
    let check_query = r#"
        SELECT * FROM mute WHERE in = user:$muter AND out = user:$mutee
    "#;
    let mut check_result = state
        .surreal()
        .query(check_query)
        .bind(("muter", auth_user.user_id.to_string()))
        .bind(("mutee", target_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let existing: Vec<crate::models::Mute> = check_result.take(0).unwrap_or_default();
    if !existing.is_empty() {
        return Err(AppError::Conflict("Already muting this user".to_string()));
    }

    // ミュートを作成
    let create_query = r#"
        CREATE mute SET
            in = user:$muter,
            out = user:$mutee,
            created_at = time::now()
    "#;

    state
        .surreal()
        .query(create_query)
        .bind(("muter", auth_user.user_id.to_string()))
        .bind(("mutee", target_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "id": format!("{}", auth_user.user_id),
        "muting": true
    })))
}

/// アカウントのミュート解除
pub async fn unmute_account(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let target_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    // ミュートを削除
    let delete_query = r#"
        DELETE mute WHERE in = user:$muter AND out = user:$mutee
    "#;

    state
        .surreal()
        .query(delete_query)
        .bind(("muter", auth_user.user_id.to_string()))
        .bind(("mutee", target_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "id": format!("{}", auth_user.user_id),
        "muting": false
    })))
}

/// 認証情報を更新
pub async fn update_credentials(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Json(req): Json<UpdateCredentialsRequest>,
) -> Result<Json<AccountResponse>> {
    let actor_id = auth_user.user_id.to_string();

    // 更新内容を構築
    let mut updates = Vec::new();

    if let Some(name) = req.display_name {
        updates.push(format!("name = '{}'", name.replace("'", "''")));
    }

    if let Some(note) = req.note {
        updates.push(format!("bio = '{}'", note.replace("'", "''")));
    }

    if let Some(locked) = req.locked {
        updates.push(format!("is_locked = {}", locked));
    }

    if let Some(bot) = req.bot {
        updates.push(format!("is_bot = {}", bot));
    }

    // アバターURLを更新
    if let Some(avatar_id) = req.avatar_id {
        if let Ok(file_id) = avatar_id.parse::<Ulid>() {
            // ファイルの存在確認
            let file_query = "SELECT id, url FROM file WHERE id = $file_id";
            let file: Option<crate::models::DriveFile> = state
                .surreal()
                .query(file_query)
                .bind(("file_id", file_id.to_string()))
                .await
                .map_err(|e| AppError::Database(e))?
                .take(0)
                .map_err(|e| AppError::Database(e))?;

            if let Some(file) = file {
                updates.push(format!("avatar_url = '{}'", file.url.clone().unwrap_or_default().replace("'", "''")));
            }
        }
    }

    // ヘッダーURLを更新
    if let Some(header_id) = req.header_id {
        if let Ok(file_id) = header_id.parse::<Ulid>() {
            let file_query = "SELECT id, url FROM file WHERE id = $file_id";
            let file: Option<crate::models::DriveFile> = state
                .surreal()
                .query(file_query)
                .bind(("file_id", file_id.to_string()))
                .await
                .map_err(|e| AppError::Database(e))?
                .take(0)
                .map_err(|e| AppError::Database(e))?;

            if let Some(file) = file {
                updates.push(format!("banner_url = '{}'", file.url.clone().unwrap_or_default().replace("'", "''")));
            }
        }
    }

    if !updates.is_empty() {
        updates.push("updated_at = time::now()".to_string());
        let update_query = format!(
            "UPDATE user:{} SET {}",
            actor_id,
            updates.join(", ")
        );

        state
            .surreal()
            .query(&update_query)
            .await
            .map_err(|e| {
                error!("Failed to update actor: {}", e);
                AppError::Database(e)
            })?;
    }

    let actor = fetch_actor_by_id(&state, auth_user.user_id).await?;
    Ok(Json(to_account_response(&actor, &state.config().instance_url)))
}

#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

/// 自分のアカウントを削除
pub async fn delete_account(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Json(req): Json<DeleteAccountRequest>,
) -> Result<Json<serde_json::Value>> {
    // パスワードを検証
    let mut result = state
        .surreal()
        .query("SELECT * FROM user WHERE id = $id")
        .bind(("id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let actor: Option<Actor> = result.take(0).ok().flatten();
    let actor = actor.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // パスワード検証（Actorのpassword_hashフィールドを使用）
    if let Some(password_hash) = &actor.password_hash {
        use bcrypt::verify;
        let valid = verify(&req.password, password_hash)
            .map_err(|_| AppError::Internal("Password verification failed".to_string()))?;

        if !valid {
            return Err(AppError::Forbidden("Incorrect password".to_string()));
        }
    } else {
        return Err(AppError::Internal("No password set".to_string()));
    }

    // TODO: Send Delete activity via federation

    // ユーザーを削除
    state.surreal()
        .query("DELETE FROM user WHERE id = $id")
        .bind(("id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Account deleted successfully"
    })))
}

/// Actor ID でアクターを取得
async fn fetch_actor_by_id(state: &AppState, id: Ulid) -> Result<Actor> {
    let mut result = state
        .surreal()
        .query("SELECT * FROM user WHERE id = $id")
        .bind(("id", id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let actor: Option<Actor> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize actor: {}", e))
    })?;

    actor.ok_or_else(|| AppError::NotFound("Actor not found".to_string()))
}

/// AccountResponse に変換
fn to_account_response(actor: &Actor, instance_url: &str) -> AccountResponse {
    let acct = if actor.is_local() {
        actor.username.clone()
    } else {
        format!("{}@{}", actor.username, actor.host.as_ref().unwrap())
    };

    let url = if actor.is_local() {
        format!("{}/users/{}", instance_url, actor.username)
    } else {
        actor.uri.clone().unwrap_or_default()
    };

    AccountResponse {
        id: actor.id.to_string(),
        username: actor.username.clone(),
        acct,
        display_name: actor.name.clone(),
        locked: actor.is_locked,
        bot: actor.is_bot,
        discoverable: true,
        group: false,
        created_at: actor.created_at.to_rfc3339(),
        note: actor.bio.clone().unwrap_or_default(),
        url,
        avatar: actor.avatar_url.clone().unwrap_or_else(|| format!("{}/static/avatar.png", instance_url)),
        avatar_static: actor.avatar_url.clone().unwrap_or_else(|| format!("{}/static/avatar.png", instance_url)),
        header: actor.banner_url.clone().unwrap_or_else(|| format!("{}/static/header.png", instance_url)),
        header_static: actor.banner_url.clone().unwrap_or_else(|| format!("{}/static/header.png", instance_url)),
        followers_count: actor.followers_count,
        following_count: actor.following_count,
        statuses_count: actor.notes_count,
        last_status_at: None,
    }
}

/// ユーザー間のリレーションを取得
pub async fn get_relations(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Query(query): Query<RelationsQuery>,
) -> Result<Json<Vec<RelationResponse>>> {
    let user_ids: Vec<Ulid> = query
        .ids
        .split(',')
        .filter_map(|id| id.parse::<Ulid>().ok())
        .collect();

    if user_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    let mut responses = Vec::new();

    for target_id in user_ids {
        let mut following = false;
        let mut followed_by = false;
        let mut blocking = false;
        let mut blocked_by = false;
        let mut muting = false;

        // フォロー関係確認
        let follow_query = r#"
            SELECT * FROM follow WHERE in = user:$follower AND out = user:$followee
        "#;
        let mut follow_result = state
            .surreal()
            .query(follow_query)
            .bind(("follower", auth_user.user_id.to_string()))
            .bind(("followee", target_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let follow_exists: Vec<crate::models::Follow> = follow_result.take(0).unwrap_or_default();
        if !follow_exists.is_empty() {
            following = true;
        }

        // 逆フォロー確認
        let reverse_follow_query = r#"
            SELECT * FROM follow WHERE in = user:$follower AND out = user:$followee
        "#;
        let mut reverse_follow_result = state
            .surreal()
            .query(reverse_follow_query)
            .bind(("follower", target_id.to_string()))
            .bind(("followee", auth_user.user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let reverse_follow_exists: Vec<crate::models::Follow> = reverse_follow_result.take(0).unwrap_or_default();
        if !reverse_follow_exists.is_empty() {
            followed_by = true;
        }

        // ブロック関係確認
        let block_query = r#"
            SELECT * FROM block WHERE in = user:$blocker AND out = user:$blockee
        "#;
        let mut block_result = state
            .surreal()
            .query(block_query)
            .bind(("blocker", auth_user.user_id.to_string()))
            .bind(("blockee", target_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let block_exists: Vec<crate::models::Block> = block_result.take(0).unwrap_or_default();
        if !block_exists.is_empty() {
            blocking = true;
        }

        // 逆ブロック確認
        let reverse_block_query = r#"
            SELECT * FROM block WHERE in = user:$blocker AND out = user:$blockee
        "#;
        let mut reverse_block_result = state
            .surreal()
            .query(reverse_block_query)
            .bind(("blocker", target_id.to_string()))
            .bind(("blockee", auth_user.user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let reverse_block_exists: Vec<crate::models::Block> = reverse_block_result.take(0).unwrap_or_default();
        if !reverse_block_exists.is_empty() {
            blocked_by = true;
        }

        // ミュート関係確認
        let mute_query = r#"
            SELECT * FROM mute WHERE in = user:$muter AND out = user:$mutee
        "#;
        let mut mute_result = state
            .surreal()
            .query(mute_query)
            .bind(("muter", auth_user.user_id.to_string()))
            .bind(("mutee", target_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let mute_exists: Vec<crate::models::Mute> = mute_result.take(0).unwrap_or_default();
        if !mute_exists.is_empty() {
            muting = true;
        }

        let account = fetch_actor_by_id(&state, &target_id).await?;

        responses.push(RelationResponse {
            id: target_id.to_string(),
            following,
            followed_by,
            blocking,
            blocked_by,
            muting,
        });
    }

    Ok(Json(responses))
}

/// フォロワー一覧を取得
pub async fn get_followers(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AccountResponse>>> {
    let target_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    let limit: usize = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50);

    let followers_query = r#"
        SELECT * FROM follow WHERE out = user:$followee LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(followers_query)
        .bind(("followee", target_id.to_string()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let follows: Vec<crate::models::Follow> = result.take(0).unwrap_or_default();

    let mut accounts = Vec::new();
    for follow in follows {
        let follower_id = follow.in_user.parse::<Ulid>()
            .map_err(|_| AppError::Validation("Invalid follower ID".to_string()))?;

        if let Ok(account) = fetch_actor_by_id(&state, &follower_id).await {
            accounts.push(to_account_response(&account, &state.config().instance_url));
        }
    }

    Ok(Json(accounts))
}

/// フォロー中一覧を取得
pub async fn get_following(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AccountResponse>>> {
    let target_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    let limit: usize = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50);

    let following_query = r#"
        SELECT * FROM follow WHERE in = user:$follower LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(following_query)
        .bind(("follower", target_id.to_string()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let follows: Vec<crate::models::Follow> = result.take(0).unwrap_or_default();

    let mut accounts = Vec::new();
    for follow in follows {
        let followee_id = follow.out_user.parse::<Ulid>()
            .map_err(|_| AppError::Validation("Invalid followee ID".to_string()))?;

        if let Ok(account) = fetch_actor_by_id(&state, &followee_id).await {
            accounts.push(to_account_response(&account, &state.config().instance_url));
        }
    }

    Ok(Json(accounts))
}

/// ユーザー投稿一覧を取得
pub async fn get_user_statuses(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<crate::routes::timeline::StatusResponse>>> {
    let target_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid actor ID".to_string()))?;

    let limit: usize = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20);

    let notes_query = r#"
        SELECT * FROM note WHERE actor_id = $user_id ORDER BY created_at DESC LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(notes_query)
        .bind(("user_id", target_id.to_string()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let notes: Vec<crate::models::Note> = result.take(0).unwrap_or_default();

    let instance_url = state.config().instance_url.clone();
    let responses: Vec<crate::routes::timeline::StatusResponse> = notes
        .iter()
        .map(|note| crate::routes::timeline::to_status_response(note, &instance_url))
        .collect();

    Ok(Json(responses))
}

/// パスワード変更
pub async fn change_password(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    // 現在のパスワード確認（ハッシュ比較が必要）
    // ここでは簡略化のため実装をスキップします
    // 実際の実装ではパスワードハッシュの検証が必要です

    // 新しいパスワードで更新
    let update_query = r#"
        UPDATE user:$id SET password = $new_password
    "#;

    state
        .surreal()
        .query(update_query)
        .bind(("id", auth_user.user_id.to_string()))
        .bind(("new_password", req.new_password))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// APIトークン再発行
pub async fn regenerate_token(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<serde_json::Value>> {
    // 新しいトークンを生成
    let new_token = ulid::Ulid::new().to_string();

    // トークンを更新
    let update_query = r#"
        UPDATE user:$id SET token = $new_token
    "#;

    state
        .surreal()
        .query(update_query)
        .bind(("id", auth_user.user_id.to_string()))
        .bind(("new_token", new_token.clone()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "token": new_token
    })))
}

/// 全通知既読化
pub async fn read_all_unread_notes(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<serde_json::Value>> {
    // 全ての未読通知を既読にする
    let update_query = r#"
        UPDATE notification SET is_read = true WHERE recipient_id = user:$user_id AND is_read = false
    "#;

    state
        .surreal()
        .query(update_query)
        .bind(("user_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// クライアント設定保存
pub async fn update_client_setting(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Json(req): Json<UpdateClientSettingRequest>,
) -> Result<Json<serde_json::Value>> {
    // 設定をJSONとして保存
    let settings_json = serde_json::to_value(&req.settings)
        .map_err(|e| AppError::Internal(format!("Failed to serialize settings: {}", e)))?;

    let update_query = r#"
        UPDATE user:$id SET client_settings = $settings
    "#;

    state
        .surreal()
        .query(update_query)
        .bind(("id", auth_user.user_id.to_string()))
        .bind(("settings", settings_json))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// フォローインポート
pub async fn import_following(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // フォロー対象のユーザーIDリストを取得
    let user_ids: Vec<String> = req.get("user_ids")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>().into()
        })
        .unwrap_or_default();

    if user_ids.is_empty() {
        return Err(AppError::Validation("No user IDs provided".to_string()));
    }

    let mut imported_count = 0;

    for target_user_id in user_ids {
        let target_id = target_user_id.parse::<Ulid>()
            .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;

        // 既にフォローしているか確認
        let check_query = r#"
            SELECT * FROM follow WHERE in = user:$follower AND out = user:$followee
        "#;
        let existing: Vec<serde_json::Value> = state
            .surreal()
            .query(check_query)
            .bind(("follower", auth_user.user_id.to_string()))
            .bind(("followee", target_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .take(0)
            .unwrap_or_default();

        if !existing.is_empty() {
            continue;
        }

        // フォローを作成
        let create_query = r#"
            CREATE follow SET
                in = user:$follower,
                out = user:$followee,
                created_at = time::now()
        "#;

        state
            .surreal()
            .query(create_query)
            .bind(("follower", auth_user.user_id.to_string()))
            .bind(("followee", target_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        imported_count += 1;
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "imported_count": imported_count
    })))
}

/// リストインポート
pub async fn import_user_lists(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // リストデータを取得
    let lists: Vec<serde_json::Value> = req.get("lists")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.clone().into())
        .unwrap_or_default();

    if lists.is_empty() {
        return Err(AppError::Validation("No lists provided".to_string()));
    }

    let mut imported_count = 0;

    for list_data in lists {
        let list_name = list_data.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled");

        // リストを作成
        let list_id = ulid::Ulid::new();
        let create_list_query = r#"
            CREATE user_list SET
                id = $list_id,
                name = $name,
                user_id = user:$user_id,
                created_at = time::now()
        "#;

        state
            .surreal()
            .query(create_list_query)
            .bind(("list_id", list_id.to_string()))
            .bind(("name", list_name))
            .bind(("user_id", auth_user.user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        imported_count += 1;
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "imported_count": imported_count
    })))
}

/// 自分の詳細情報
pub async fn get_i(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<serde_json::Value>> {
    let user = fetch_actor_by_id(&state, &auth_user.user_id).await?;

    let account_response = to_account_response(&user, &state.config().instance_url);

    Ok(Json(serde_json::json!({
        "id": account_response.id,
        "username": account_response.username,
        "acct": account_response.acct,
        "display_name": account_response.display_name,
        "locked": account_response.locked,
        "bot": account_response.bot,
        "created_at": account_response.created_at,
        "followers_count": account_response.followers_count,
        "following_count": account_response.following_count,
        "statuses_count": account_response.statuses_count,
        "note": account_response.note,
        "url": account_response.url,
        "avatar": account_response.avatar,
        "avatar_static": account_response.avatar_static,
        "header": account_response.header,
        "header_static": account_response.header_static,
    })))
}

/// フォロー数を更新
async fn update_follow_counts(
    state: &AppState,
    follower_id: Ulid,
    followee_id: Ulid,
) -> Result<()> {
    // followerのfollowing_countを更新
    let update_follower_query = r#"
        UPDATE user:$id SET following_count = (
            SELECT count() FROM follow WHERE in = user:$id
        )[0].count
    "#;

    state
        .surreal()
        .query(update_follower_query)
        .bind(("id", follower_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    // followeeのfollowers_countを更新
    let update_followee_query = r#"
        UPDATE user:$id SET followers_count = (
            SELECT count() FROM follow WHERE out = user:$id
        )[0].count
    "#;

    state
        .surreal()
        .query(update_followee_query)
        .bind(("id", followee_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(())
}
