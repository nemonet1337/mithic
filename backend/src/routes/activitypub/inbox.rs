use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{Actor, Note, NoteVisibility, Reaction},
    state::AppState,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Activity {
    #[serde(rename = "@context")]
    pub context: Option<serde_json::Value>,
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    pub object: Option<serde_json::Value>,
    pub target: Option<String>,
}

/// Inboxエンドポイント
pub async fn inbox(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(activity): Json<Activity>,
) -> Result<Json<serde_json::Value>> {
    info!(
        "Received Activity {} for user {}: type={}",
        activity.id, username, activity.activity_type
    );

    // ローカルユーザーの存在確認
    let mut result = state
        .surreal()
        .query("SELECT id FROM user WHERE username_lower = $username")
        .bind(("username", username.to_lowercase()))
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            AppError::Database(e)
        })?;

    let user_exists: Option<String> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize: {}", e))
    })?;

    if user_exists.is_none() {
        return Err(AppError::NotFound(crate::t!("actor-not-found")));
    }

    // アクティビティタイプに応じた処理
    match activity.activity_type.as_str() {
        "Create" => {
            handle_create(&state, &activity).await?;
        }
        "Delete" => {
            handle_delete(&state, &activity).await?;
        }
        "Follow" => {
            handle_follow(&state, &username, &activity).await?;
        }
        "Undo" => {
            handle_undo(&state, &username, &activity).await?;
        }
        "Accept" => {
            handle_accept(&state, &activity).await?;
        }
        "Reject" => {
            handle_reject(&state, &activity).await?;
        }
        "Announce" => {
            handle_announce(&state, &activity).await?;
        }
        "Like" => {
            handle_like(&state, &activity).await?;
        }
        "Update" => {
            handle_update(&state, &activity).await?;
        }
        _ => {
            warn!("Unknown activity type: {}", activity.activity_type);
        }
    }

    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// SharedInboxエンドポイント（インスタンス全体のInbox）
/// Broadcast activities (Create, Delete, Announce, Like, Update)を効率的に受信
pub async fn shared_inbox(
    State(state): State<AppState>,
    Json(activity): Json<Activity>,
) -> Result<Json<serde_json::Value>> {
    info!(
        "Received Activity in sharedInbox: type={} from={}",
        activity.activity_type, activity.actor
    );

    // アクティビティタイプに応じた処理
    // Follow/Undo(Follow)は個別Inboxのみで処理（target指定が必要）
    match activity.activity_type.as_str() {
        "Create" => {
            handle_create(&state, &activity).await?;
        }
        "Delete" => {
            handle_delete(&state, &activity).await?;
        }
        "Announce" => {
            handle_announce(&state, &activity).await?;
        }
        "Like" => {
            handle_like(&state, &activity).await?;
        }
        "Update" => {
            handle_update(&state, &activity).await?;
        }
        "Accept" | "Reject" => {
            // フォロー承認/拒否はactorから辿って処理
            handle_accept(&state, &activity).await?;
        }
        _ => {
            // Follow/Undoは個別inboxで処理すべき
            warn!("Activity type {} should use individual inbox, not sharedInbox", activity.activity_type);
        }
    }

    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// Createアクティビティの処理
async fn handle_create(state: &AppState, activity: &Activity) -> Result<()> {
    info!("Processing Create activity from {}", activity.actor);

    let object = activity.object.as_ref()
        .ok_or_else(|| AppError::Validation("Missing object in Create activity".to_string()))?;

    let obj_type = object.get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown");

    match obj_type {
        "Note" => {
            // リモートノートの受信
            let note_data = object.clone();
            let content = note_data.get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();

            let actor_uri = note_data.get("attributedTo")
                .and_then(|a| a.as_str())
                .unwrap_or(&activity.actor)
                .to_string();

            let note_id = note_data.get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string();

            // リモートActorを取得または作成
            let remote_actor = fetch_or_create_remote_actor(state, &actor_uri).await?;

            // ノートを作成
            let mut note = Note::new(remote_actor.id, Some(content), NoteVisibility::Public);
            note.uri = Some(note_id);

            // SurrealDBに保存
            state.surreal()
                .create(("note", note.id.to_string()))
                .content(note)
                .await
                .map_err(|e| {
                    error!("Failed to create note: {}", e);
                    AppError::Database(e)
                })?;

            info!("Created remote note from {}", actor_uri);
        }
        _ => {
            warn!("Create activity for unknown object type: {}", obj_type);
        }
    }

    Ok(())
}

/// Deleteアクティビティの処理
async fn handle_delete(state: &AppState, activity: &Activity) -> Result<()> {
    info!("Processing Delete activity from {}", activity.actor);

    if let Some(object) = &activity.object {
        if let Some(obj_id) = object.get("id").and_then(|i| i.as_str()) {
            // URIでノートを検索して削除
            let delete_query = "DELETE note WHERE uri = $uri";
            state.surreal()
                .query(delete_query)
                .bind(("uri", obj_id.to_string()))
                .await
                .map_err(|e| AppError::Database(e))?;

            info!("Deleted note with URI: {}", obj_id);
        }
    }

    Ok(())
}

/// Followアクティビティの処理
async fn handle_follow(state: &AppState, target_username: &str, activity: &Activity) -> Result<()> {
    info!("Processing Follow activity from {} to {}", activity.actor, target_username);

    // フォロー元Actorを取得または作成
    let follower = fetch_or_create_remote_actor(state, &activity.actor).await?;

    // ターゲットユーザーを取得
    let mut result = state
        .surreal()
        .query("SELECT id FROM user WHERE username_lower = $username")
        .bind(("username", target_username.to_lowercase()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let followee_id: Option<String> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize: {}", e))
    })?;

    let followee_id = followee_id
        .ok_or_else(|| AppError::NotFound(crate::t!("actor-not-found")))?;

    // ターゲットユーザーの情報を取得（承認制・鍵の確認）
    let followee_query = r#"
        SELECT is_locked, inbox, public_key, private_key FROM user WHERE id = $followee_id
    "#;
    let mut followee_result = state
        .surreal()
        .query(followee_query)
        .bind(("followee_id", followee_id.clone()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let followee_data: Option<(bool, Option<String>, Option<String>, Option<String>)> = followee_result
        .take(0)
        .map_err(|e| AppError::Internal(format!("Failed to deserialize: {}", e)))?;

    let (is_locked, followee_inbox, _public_key, private_key) = followee_data
        .ok_or_else(|| AppError::NotFound(crate::t!("actor-not-found")))?;

    // フォロー関係を作成（承認制でない場合は即承認）
    let is_accepted = !is_locked;
    let relation_query = r#"
        RELATE user:$follower->follow->user:$followee
        SET created_at = time::now(), is_accepted = $is_accepted
    "#;

    state.surreal()
        .query(relation_query)
        .bind(("follower", follower.id.to_string()))
        .bind(("followee", followee_id.clone()))
        .bind(("is_accepted", is_accepted))
        .await
        .map_err(|e| {
            error!("Failed to create follow relation: {}", e);
            AppError::Database(e)
        })?;

    // フォロー数を更新
    update_follow_counts(state, &follower.id.to_string(), &followee_id).await?;

    info!("Created follow relation: {} -> {} (accepted: {})", follower.username, target_username, is_accepted);

    // 自動承認の場合、Acceptアクティビティを返送
    if is_accepted {
        if let (Some(inbox), Some(_pk)) = (followee_inbox, private_key) {
            // FederationServiceを作成してAcceptを配送
            use crate::services::federation::FederationService;
            use crate::db::{SurrealClient, DragonflyClient};

            let federation = FederationService::new(
                SurrealClient::from_ref(state),
                DragonflyClient::from_ref(state),
                state.config().instance_url.clone(),
            );

            // ローカルアクター情報を構築
            let local_actor = crate::models::Actor {
                id: ulid::Ulid::from_string(&followee_id).unwrap_or_default(),
                username: target_username.to_string(),
                name: None,
                username_lower: target_username.to_lowercase(),
                bio: None,
                password_hash: None,
                email: None,
                created_at: chrono::Utc::now(),
                updated_at: None,
                followers_count: 0,
                following_count: 0,
                notes_count: 0,
                avatar_url: None,
                banner_url: None,
                is_suspended: false,
                is_locked,
                is_bot: false,
                is_admin: false,
                host: None,
                token: None,
                inbox: Some(inbox),
                shared_inbox: Option::<String>::None,
                featured: None,
                uri: Some(format!("{}/users/{}", state.config().instance_url, target_username)),
                public_key: _public_key,
                private_key: Some(_pk),
            };

            let follower_inbox = follower.inbox.clone()
                .unwrap_or_else(|| format!("{}/inbox", follower.uri.unwrap_or_default()));

            if let Err(e) = federation.send_accept_follow(
                &follower_inbox,
                &follower.uri.unwrap_or_default(),
                &local_actor,
            ).await {
                warn!("Failed to send Accept activity: {}", e);
                // Accept送信失敗はフォロー関係には影響させない
            }
        }
    }

    Ok(())
}

/// Undoアクティビティの処理
async fn handle_undo(state: &AppState, target_username: &str, activity: &Activity) -> Result<()> {
    info!("Processing Undo activity from {}", activity.actor);

    if let Some(object) = &activity.object {
        let obj_type = object.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown");

        match obj_type {
            "Follow" => {
                // フォロー解除
                if let Some(actor_url) = object.get("actor").and_then(|a| a.as_str()) {
                    let follower = fetch_or_create_remote_actor(state, actor_url).await?;

                    let delete_query = r#"
                        DELETE follow WHERE in = user:$follower AND out = user:$followee
                    "#;

                    state.surreal()
                        .query(delete_query)
                        .bind(("follower", follower.id.to_string()))
                        .bind(("followee", target_username.to_lowercase()))
                        .await
                        .map_err(|e| AppError::Database(e))?;

                    info!("Deleted follow relation: {} -> {}", actor_url, target_username);
                }
            }
            "Like" => {
                // お気に入り解除
                warn!("Undo Like not fully implemented");
            }
            "Announce" => {
                // ブースト解除
                warn!("Undo Announce not fully implemented");
            }
            _ => {
                warn!("Undo for unknown object type: {}", obj_type);
            }
        }
    }

    Ok(())
}

/// Acceptアクティビティの処理
async fn handle_accept(state: &AppState, activity: &Activity) -> Result<()> {
    info!("Processing Accept activity from {}", activity.actor);

    if let Some(object) = &activity.object {
        let obj_type = object.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown");

        if obj_type == "Follow" {
            // フォロー承認を受信 - following関係のis_acceptedをtrueに更新
            // object.actorがフォロー元（自分）、object.objectがフォロー先（相手）
            if let (Some(actor_url), Some(target_url)) = (
                object.get("actor").and_then(|a| a.as_str()),
                object.get("object").and_then(|o| o.as_str())
            ) {
                // 自分がフォロー元か確認（自分からのフォローリクエストの承認）
                let local_actor_result: Option<Actor> = state
                    .surreal()
                    .query("SELECT * FROM user WHERE uri = $uri LIMIT 1")
                    .bind(("uri", actor_url))
                    .await
                    .and_then(|mut res| res.take(0))
                    .ok()
                    .flatten();

                if local_actor_result.is_some() {
                    // 自分のfollowing関係を更新
                    let update_query = r#"
                        UPDATE follow 
                        SET is_accepted = true 
                        WHERE in = (SELECT id FROM user WHERE uri = $actor_url)
                        AND out = (SELECT id FROM user WHERE uri = $target_url)
                    "#;

                    state.surreal()
                        .query(update_query)
                        .bind(("actor_url", actor_url))
                        .bind(("target_url", target_url))
                        .await
                        .map_err(|e| {
                            error!("Failed to update follow relation: {}", e);
                            AppError::Database(e)
                        })?;

                    info!("Updated follow relation to accepted: {} -> {}", actor_url, target_url);
                }
            }
        }
    }

    Ok(())
}

/// Rejectアクティビティの処理
async fn handle_reject(state: &AppState, activity: &Activity) -> Result<()> {
    info!("Processing Reject activity from {}", activity.actor);

    if let Some(object) = &activity.object {
        let obj_type = object.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown");

        if obj_type == "Follow" {
            // フォロー拒否を受信 - following関係を削除
            if let (Some(actor_url), Some(target_url)) = (
                object.get("actor").and_then(|a| a.as_str()),
                object.get("object").and_then(|o| o.as_str())
            ) {
                // 自分がフォロー元か確認（自分からのフォローリクエストの拒否）
                let local_actor_result: Option<Actor> = state
                    .surreal()
                    .query("SELECT * FROM user WHERE uri = $uri LIMIT 1")
                    .bind(("uri", actor_url))
                    .await
                    .and_then(|mut res| res.take(0))
                    .ok()
                    .flatten();

                if local_actor_result.is_some() {
                    // 自分のfollowing関係を削除
                    let delete_query = r#"
                        DELETE follow 
                        WHERE in = (SELECT id FROM user WHERE uri = $actor_url)
                        AND out = (SELECT id FROM user WHERE uri = $target_url)
                    "#;

                    state.surreal()
                        .query(delete_query)
                        .bind(("actor_url", actor_url))
                        .bind(("target_url", target_url))
                        .await
                        .map_err(|e| {
                            error!("Failed to delete follow relation: {}", e);
                            AppError::Database(e)
                        })?;

                    info!("Deleted follow relation due to reject: {} -> {}", actor_url, target_url);
                }
            }
        }
    }

    Ok(())
}

/// Announceアクティビティの処理（ブースト/リノート）
async fn handle_announce(state: &AppState, activity: &Activity) -> Result<()> {
    info!("Processing Announce activity from {}", activity.actor);

    if let Some(object) = &activity.object {
        let announced_uri = if let Some(uri) = object.as_str() {
            uri.to_string()
        } else if let Some(id) = object.get("id").and_then(|i| i.as_str()) {
            id.to_string()
        } else {
            return Err(AppError::Validation("Missing announced object URI".to_string()));
        };

        // アナウンス元Actorを取得または作成
        let announcer = fetch_or_create_remote_actor(state, &activity.actor).await?;

        // ブースト用ノートを作成
        let mut note = Note::new(announcer.id, None, NoteVisibility::Public);
        note.uri = Some(activity.id.clone());
        note.renote_uri = Some(announced_uri);

        state.surreal()
            .create(("note", note.id.to_string()))
            .content(note)
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created announce note from {}", activity.actor);
    }

    Ok(())
}

/// Likeアクティビティの処理（お気に入り）
async fn handle_like(state: &AppState, activity: &Activity) -> Result<()> {
    info!("Processing Like activity from {}", activity.actor);

    if let Some(object) = &activity.object {
        let liked_uri = if let Some(uri) = object.as_str() {
            uri.to_string()
        } else if let Some(id) = object.get("id").and_then(|i| i.as_str()) {
            id.to_string()
        } else {
            return Err(AppError::Validation("Missing liked object URI".to_string()));
        };

        // リアクション元Actorを取得または作成
        let actor = fetch_or_create_remote_actor(state, &activity.actor).await?;

        // URIでノートを検索
        let note_result: Option<Note> = state
            .surreal()
            .query("SELECT * FROM note WHERE uri = $uri LIMIT 1")
            .bind(("uri", liked_uri.clone()))
            .await
            .and_then(|mut res| res.take(0))
            .ok()
            .flatten();

        let note = match note_result {
            Some(n) => n,
            None => {
                warn!("Note not found for Like activity: {}", liked_uri);
                return Ok(());
            }
        };

        // 重複チェック
        let existing: Option<Reaction> = state
            .surreal()
            .query("SELECT * FROM reaction WHERE note_id = $note_id AND actor_id = $actor_id LIMIT 1")
            .bind(("note_id", note.id.to_string()))
            .bind(("actor_id", actor.id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .ok()
            .flatten();

        if existing.is_some() {
            info!("Reaction already exists for note {} by actor {}", note.id, actor.id);
            return Ok(());
        }

        // リアクションを作成（リモート）
        let reaction = Reaction::new_remote(
            note.id,
            actor.id,
            "⭐".to_string(), // ActivityPub Likeは通常⭐として扱う
            activity.id.clone(),
        );

        state.surreal()
            .create(("reaction", reaction.id.to_string()))
            .content(reaction.clone())
            .await
            .map_err(|e| {
                error!("Failed to create reaction: {}", e);
                AppError::Database(e)
            })?;

        // ノートのリアクションカウントを更新
        state.surreal()
            .query(r#"
                UPDATE note 
                SET reactions['⭐'] = (reactions['⭐'] ?? 0) + 1
                WHERE id = $note_id
            "#)
            .bind(("note_id", note.id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created remote reaction {} for note {} by actor {}", reaction.id, note.id, actor.id);
    }

    Ok(())
}

/// Updateアクティビティの処理
async fn handle_update(state: &AppState, activity: &Activity) -> Result<()> {
    info!("Processing Update activity from {}", activity.actor);

    if let Some(object) = &activity.object {
        let obj_type = object.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown");

        match obj_type {
            "Person" | "Service" | "Application" => {
                // Actor更新
                let actor_uri = object.get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or(&activity.actor);

                let _ = fetch_or_create_remote_actor(state, actor_uri).await?;
                info!("Updated remote actor: {}", actor_uri);
            }
            "Note" => {
                // ノート更新
                warn!("Note update not fully implemented");
            }
            _ => {
                warn!("Update for unknown object type: {}", obj_type);
            }
        }
    }

    Ok(())
}

/// リモートActorを取得または作成
async fn fetch_or_create_remote_actor(state: &AppState, actor_uri: &str) -> Result<Actor> {
    // 既存のActorを検索
    let mut result = state
        .surreal()
        .query("SELECT * FROM user WHERE uri = $uri")
        .bind(("uri", actor_uri.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let existing: Option<Actor> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize: {}", e))
    })?;

    if let Some(actor) = existing {
        return Ok(actor);
    }

    // 新規作成 - リモートActorを取得
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(actor_uri)
        .header("Accept", "application/activity+json, application/ld+json")
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch actor: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "Actor fetch failed with status: {}",
            response.status()
        )));
    }

    let actor_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse actor JSON: {}", e)))?;

    // Actor情報を抽出
    let username = actor_data.get("preferredUsername")
        .and_then(|u| u.as_str())
        .unwrap_or("unknown")
        .to_string();

    let name = actor_data.get("name")
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());

    let bio = actor_data.get("summary")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    let inbox_url = actor_data.get("inbox")
        .and_then(|i| i.as_str())
        .map(|s| s.to_string());

    let shared_inbox = actor_data.get("endpoints")
        .and_then(|e| e.get("sharedInbox"))
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    let avatar_url = actor_data.get("icon")
        .and_then(|i| i.get("url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    // Hostを抽出
    let host = extract_host_from_uri(actor_uri);

    // Actorを作成
    let mut actor = Actor {
        id: Ulid::new(),
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        username: username.clone(),
        username_lower: username.to_lowercase(),
        name,
        bio,
        followers_count: 0,
        following_count: 0,
        notes_count: 0,
        avatar_url,
        banner_url: None,
        is_suspended: false,
        is_locked: false,
        is_bot: false,
        is_admin: false,
        host: Some(host),
        inbox: inbox_url,
        shared_inbox,
        featured: None,
        uri: Some(actor_uri.to_string()),
        public_key: actor_data.get("publicKey")
            .and_then(|pk| pk.get("publicKeyPem"))
            .and_then(|p| p.as_str())
            .map(|s| s.to_string()),
        private_key: None,
        token: None,
        password_hash: None,
        email: None,
    };

    // SurrealDBに保存
    state.surreal()
        .create(("user", actor.id.to_string()))
        .content(actor.clone())
        .await
        .map_err(|e| AppError::Database(e))?;

    info!("Created remote actor: {}@{}", actor.username, actor.host.as_ref().unwrap_or(&"unknown".to_string()));

    Ok(actor)
}

/// URIからHostを抽出
fn extract_host_from_uri(uri: &str) -> String {
    uri.split('/')
        .nth(2)
        .unwrap_or("unknown")
        .to_string()
}

/// フォロー数を更新
async fn update_follow_counts(state: &AppState, follower_id: &str, followee_id: &str) -> Result<()> {
    // followerのfollowing_countを更新
    let update_follower_query = r#"
        UPDATE user:$id SET following_count = (
            SELECT count() FROM follow WHERE in = user:$id
        )[0].count
    "#;

    state.surreal()
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

    state.surreal()
        .query(update_followee_query)
        .bind(("id", followee_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(())
}
