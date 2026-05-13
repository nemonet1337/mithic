use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{Note, NoteVisibility},
    state::{AppState, AuthUser},
};
use crate::routes::timeline::StatusResponse;

#[derive(Debug, Deserialize)]
pub struct ConversationQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RepliesQuery {
    pub limit: Option<usize>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenotesQuery {
    pub limit: Option<usize>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MentionsQuery {
    pub limit: Option<usize>,
    pub following: Option<bool>,
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChildrenQuery {
    pub limit: Option<usize>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReactQuery {
    pub emoji: String,
}

#[derive(Debug, Deserialize)]
pub struct FavoritesQuery {
    pub limit: Option<usize>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStatusRequest {
    pub status: Option<String>,
    #[serde(rename = "in_reply_to_id")]
    pub in_reply_to_id: Option<String>,
    pub sensitive: Option<bool>,
    pub spoiler_text: Option<String>,
    pub visibility: Option<String>,
    pub scheduled_at: Option<String>,
}

/// 投稿作成
pub async fn create_status(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Json(req): Json<CreateStatusRequest>,
) -> Result<Json<StatusResponse>> {
    let visibility = req.visibility.as_ref().map(|v| {
        match v.as_str() {
            "public" => NoteVisibility::Public,
            "unlisted" => NoteVisibility::Home,
            "private" => NoteVisibility::Followers,
            "direct" => NoteVisibility::Specified,
            _ => NoteVisibility::Public,
        }
    }).unwrap_or(NoteVisibility::Public);

    let mut note = Note::new(auth_user.user_id, req.status, visibility);

    // リプライ先設定
    if let Some(reply_id_str) = req.in_reply_to_id {
        let reply_id = reply_id_str.parse::<Ulid>()
            .map_err(|_| AppError::Validation("Invalid reply ID".to_string()))?;
        note.reply_id = Some(reply_id);
    }

    // CW設定
    if req.sensitive.unwrap_or(false) || req.spoiler_text.is_some() {
        note.cw = req.spoiler_text;
    }

    // SurrealDBに保存
    let created: Note = state
        .surreal()
        .create(("note", note.id.to_string()))
        .content(note)
        .await
        .map_err(|e| {
            error!("Failed to create note: {}", e);
            AppError::Database(e)
        })?;

    // ユーザーの投稿数を更新
    let update_count_query = r#"
        UPDATE user:$id SET notes_count = notes_count + 1
    "#;
    state
        .surreal()
        .query(update_count_query)
        .bind(("id", auth_user.user_id.to_string()))
        .await
        .ok(); // エラーは無視

    Ok(Json(to_status_response(&created, &state.config().instance_url)))
}

/// 投稿取得
pub async fn get_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    let mut result = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let note: Option<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize note: {}", e))
    })?;

    let note = note.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    Ok(Json(to_status_response(&note, &state.config().instance_url)))
}

/// 投稿削除
pub async fn delete_status(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // 所有者確認
    let mut result = state
        .surreal()
        .query("SELECT actor_id FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let actor_id: Option<String> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize: {}", e))
    })?;

    let actor_id = actor_id.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    if actor_id != auth_user.user_id.to_string() {
        return Err(AppError::Forbidden("Cannot delete other user's note".to_string()));
    }

    // 削除実行
    state
        .surreal()
        .query("DELETE note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({"success": true})))
}

/// お気に入り追加（リアクション）
pub async fn favourite_status(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // ノート存在確認
    let mut result = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let note: Option<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize note: {}", e))
    })?;

    let note = note.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    // 既存リアクション確認
    let check_query = r#"
        SELECT * FROM reaction WHERE note_id = $note_id AND actor_id = $actor_id
    "#;
    let mut check_result = state
        .surreal()
        .query(check_query)
        .bind(("note_id", note_id.to_string()))
        .bind(("actor_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let existing: Option<crate::models::Reaction> = check_result.take(0).ok().flatten();

    if existing.is_some() {
        // 既にリアクション済み - そのまま返す
        return Ok(Json(to_status_response(&note, &state.config().instance_url)));
    }

    // リアクション作成（デフォルトは「like」）
    let reaction = crate::models::Reaction::new(
        note_id,
        auth_user.user_id,
        "⭐".to_string(), // デフォルトリアクション
    );

    state
        .surreal()
        .create(("reaction", reaction.id.to_string()))
        .content(reaction)
        .await
        .map_err(|e| AppError::Database(e))?;

    // ノートのリアクション数を更新
    let update_query = r#"
        UPDATE note:$note_id SET reactions['⭐'] = reactions['⭐'] + 1
    "#;
    state
        .surreal()
        .query(update_query)
        .bind(("note_id", note_id.to_string()))
        .await
        .ok(); // エラーは無視

    // 更新後のノートを取得
    let mut result = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let updated_note: Option<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize note: {}", e))
    })?;

    let updated_note = updated_note.unwrap_or(note);

    Ok(Json(to_status_response(&updated_note, &state.config().instance_url)))
}

/// お気に入り削除（リアクション解除）
pub async fn unfavourite_status(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // ノート存在確認
    let mut result = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let note: Option<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize note: {}", e))
    })?;

    let note = note.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    // リアクション削除
    let delete_query = r#"
        DELETE reaction WHERE note_id = $note_id AND actor_id = $actor_id
    "#;
    state
        .surreal()
        .query(delete_query)
        .bind(("note_id", note_id.to_string()))
        .bind(("actor_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    // ノートのリアクション数を更新（0未満にならないように）
    let update_query = r#"
        UPDATE note:$note_id SET reactions['⭐'] = reactions['⭐'] - 1
    "#;
    state
        .surreal()
        .query(update_query)
        .bind(("note_id", note_id.to_string()))
        .await
        .ok(); // エラーは無視

    Ok(Json(to_status_response(&note, &state.config().instance_url)))
}

/// ブースト/リノート
pub async fn reblog_status(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // 対象ノート存在確認
    let mut result = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let target_note: Option<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize note: {}", e))
    })?;

    let target_note = target_note.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    // 既存リノート確認
    let check_query = r#"
        SELECT * FROM renote WHERE note_id = $note_id AND actor_id = $actor_id
    "#;
    let mut check_result = state
        .surreal()
        .query(check_query)
        .bind(("note_id", note_id.to_string()))
        .bind(("actor_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let existing: Option<crate::models::Renote> = check_result.take(0).ok().flatten();

    if let Some(renote) = existing {
        // 既にリノート済み - 該当ノートを返す
        let mut result = state
            .surreal()
            .query("SELECT * FROM note WHERE id = $id")
            .bind(("id", renote.renote_note_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let renote_note: Option<Note> = result.take(0).map_err(|e| {
            AppError::Internal(format!("Failed to deserialize note: {}", e))
        })?;

        if let Some(note) = renote_note {
            return Ok(Json(to_status_response(&note, &state.config().instance_url)));
        }
    }

    // リノート用ノートを作成
    let renote_note = Note::new(
        auth_user.user_id,
        None, // リノートは本文なし
        target_note.visibility,
    );
    let renote_note_id = renote_note.id;

    // リノートノートを保存
    let created_note: Note = state
        .surreal()
        .create(("note", renote_note_id.to_string()))
        .content(renote_note)
        .await
        .map_err(|e| AppError::Database(e))?;

    // リノート関係を記録
    let renote = crate::models::Renote::new(note_id, auth_user.user_id, renote_note_id);
    state
        .surreal()
        .create(("renote", renote.id.to_string()))
        .content(renote)
        .await
        .map_err(|e| AppError::Database(e))?;

    // 対象ノートのリノート数を更新
    let update_query = r#"
        UPDATE note:$note_id SET renote_count = renote_count + 1
    "#;
    state
        .surreal()
        .query(update_query)
        .bind(("note_id", note_id.to_string()))
        .await
        .ok();

    // ユーザーの投稿数を更新
    let update_count_query = r#"
        UPDATE user:$id SET notes_count = notes_count + 1
    "#;
    state
        .surreal()
        .query(update_count_query)
        .bind(("id", auth_user.user_id.to_string()))
        .await
        .ok();

    Ok(Json(to_status_response(&created_note, &state.config().instance_url)))
}

/// ブースト/リノート解除
pub async fn unreblog_status(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // リノート関係を取得
    let check_query = r#"
        SELECT * FROM renote WHERE note_id = $note_id AND actor_id = $actor_id
    "#;
    let mut check_result = state
        .surreal()
        .query(check_query)
        .bind(("note_id", note_id.to_string()))
        .bind(("actor_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let existing: Option<crate::models::Renote> = check_result.take(0).ok().flatten();

    if let Some(renote) = existing {
        // リノートノートを削除
        state
            .surreal()
            .query("DELETE note WHERE id = $id")
            .bind(("id", renote.renote_note_id.to_string()))
            .await
            .ok();

        // リノート関係を削除
        state
            .surreal()
            .query("DELETE renote WHERE id = $id")
            .bind(("id", renote.id.to_string()))
            .await
            .ok();

        // 対象ノートのリノート数を更新
        let update_query = r#"
            UPDATE note:$note_id SET renote_count = renote_count - 1
        "#;
        state
            .surreal()
            .query(update_query)
            .bind(("note_id", note_id.to_string()))
            .await
            .ok();

        // ユーザーの投稿数を更新
        let update_count_query = r#"
            UPDATE user:$id SET notes_count = notes_count - 1
        "#;
        state
            .surreal()
            .query(update_count_query)
            .bind(("id", auth_user.user_id.to_string()))
            .await
            .ok();
    }

    Ok(Json(serde_json::json!({"success": true})))
}

/// 投稿の文脈（返信元を遡る）を取得
pub async fn get_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ConversationQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);

    // まず指定されたノートを取得
    let mut result = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let note: Option<Note> = result.take(0).ok().flatten();
    let note = note.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    // 返信元を再帰的に遡る
    let mut conversation = Vec::new();
    let mut current_reply_id = note.reply_id;
    let mut skipped = 0;

    while let Some(reply_id) = current_reply_id {
        let mut result = state
            .surreal()
            .query("SELECT * FROM note WHERE id = $id")
            .bind(("id", reply_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        if let Ok(Some(reply_note)) = result.take::<Option<Note>>(0) {
            if skipped >= offset {
                conversation.push(to_status_response(&reply_note, &state.config().instance_url));
                if conversation.len() >= limit {
                    break;
                }
            } else {
                skipped += 1;
            }
            current_reply_id = reply_note.reply_id;
        } else {
            break;
        }
    }

    Ok(Json(conversation))
}

/// 投稿への返信一覧を取得
pub async fn get_replies(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<RepliesQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    let limit = query.limit.unwrap_or(10);

    let mut surreal_query = state
        .surreal()
        .query("SELECT * FROM note WHERE reply_id = $reply_id LIMIT $limit")
        .bind(("reply_id", note_id.to_string()))
        .bind(("limit", limit));

    let mut result = surreal_query.await.map_err(|e| AppError::Database(e))?;

    let replies: Vec<Note> = result.take(0).unwrap_or_default();

    let responses: Vec<StatusResponse> = replies
        .iter()
        .map(|note| to_status_response(note, &state.config().instance_url))
        .collect();

    Ok(Json(responses))
}

/// 投稿のRenote一覧を取得
pub async fn get_renotes(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<RenotesQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    let limit = query.limit.unwrap_or(10);

    // renoteテーブルから対象ノートのRenoteを取得
    let mut result = state
        .surreal()
        .query("SELECT * FROM renote WHERE target_note_id = $target_note_id LIMIT $limit")
        .bind(("target_note_id", note_id.to_string()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let renotes: Vec<crate::models::Renote> = result.take(0).unwrap_or_default();

    // Renote元のノートを取得して返す
    let mut responses = Vec::new();
    for renote in renotes {
        let mut note_result = state
            .surreal()
            .query("SELECT * FROM note WHERE id = $id")
            .bind(("id", renote.renote_note_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        if let Ok(Some(note)) = note_result.take::<Option<Note>>(0) {
            responses.push(to_status_response(&note, &state.config().instance_url));
        }
    }

    Ok(Json(responses))
}

/// 自分へのメンション一覧を取得
pub async fn get_mentions(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Query(query): Query<MentionsQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let limit = query.limit.unwrap_or(10);

    let mut query_str = "SELECT * FROM note WHERE $user_id IN mentions LIMIT $limit".to_string();

    let mut result = state
        .surreal()
        .query(&query_str)
        .bind(("user_id", auth_user.user_id.to_string()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let mentions: Vec<Note> = result.take(0).unwrap_or_default();

    let responses: Vec<StatusResponse> = mentions
        .iter()
        .map(|note| to_status_response(note, &state.config().instance_url))
        .collect();

    Ok(Json(responses))
}

/// 投稿への返信・引用を取得
pub async fn get_children(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ChildrenQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    let limit = query.limit.unwrap_or(10);

    // 返信と引用を取得
    let query_str = r#"
        SELECT * FROM note WHERE reply_id = $note_id OR renote_id = $note_id LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(query_str)
        .bind(("note_id", note_id.to_string()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let children: Vec<Note> = result.take(0).unwrap_or_default();

    let responses: Vec<StatusResponse> = children
        .iter()
        .map(|note| to_status_response(note, &state.config().instance_url))
        .collect();

    Ok(Json(responses))
}

/// お気に入り投稿一覧を取得
pub async fn get_favorites(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Query(query): Query<FavoritesQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let limit = query.limit.unwrap_or(10);

    let mut query_str = "SELECT * FROM favorite WHERE user_id = $user_id".to_string();

    if let Some(since_id) = &query.since_id {
        query_str.push_str(&format!(" AND id > '{}'", since_id));
    }

    if let Some(until_id) = &query.until_id {
        query_str.push_str(&format!(" AND id < '{}'", until_id));
    }

    query_str.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));

    let mut result = state
        .surreal()
        .query(&query_str)
        .bind(("user_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let favorites: Vec<crate::models::Favorite> = result.take(0).unwrap_or_default();

    // お気に入りされたノートを取得
    let mut responses = Vec::new();
    for favorite in favorites {
        let mut note_result = state
            .surreal()
            .query("SELECT * FROM note WHERE id = $id")
            .bind(("id", favorite.note_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        if let Ok(Some(note)) = note_result.take::<Option<Note>>(0) {
            responses.push(to_status_response(&note, &state.config().instance_url));
        }
    }

    Ok(Json(responses))
}

/// 投稿をピン留め
pub async fn pin_note(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // 投稿の所有者確認
    let note: Option<Note> = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    let note = note.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    if note.actor_id != auth_user.user_id {
        return Err(AppError::Forbidden("Cannot pin other user's note".to_string()));
    }

    // ピン留めを追加
    state
        .surreal()
        .query("UPDATE note:$id SET is_pinned = true")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(to_status_response(&note, &state.config().instance_url)))
}

/// ピン留めを解除
pub async fn unpin_note(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // 投稿の所有者確認
    let note: Option<Note> = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    let note = note.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    if note.actor_id != auth_user.user_id {
        return Err(AppError::Forbidden("Cannot unpin other user's note".to_string()));
    }

    // ピン留めを解除
    state
        .surreal()
        .query("UPDATE note:$id SET is_pinned = false")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(to_status_response(&note, &state.config().instance_url)))
}

/// リアクションを付ける
pub async fn react_note(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
    Query(query): Query<ReactQuery>,
) -> Result<Json<StatusResponse>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // 投稿の存在確認
    let note: Option<Note> = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    let note = note.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    // 既に同じリアクションがあるか確認
    let check_query = r#"
        SELECT * FROM note_reaction WHERE note_id = $note_id AND user_id = $user_id AND emoji = $emoji
    "#;
    let existing: Vec<serde_json::Value> = state
        .surreal()
        .query(check_query)
        .bind(("note_id", note_id.to_string()))
        .bind(("user_id", auth_user.user_id.to_string()))
        .bind(("emoji", query.emoji.clone()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    if !existing.is_empty() {
        return Err(AppError::Conflict("Already reacted with this emoji".to_string()));
    }

    // リアクションを作成
    let create_query = r#"
        CREATE note_reaction SET
            note_id = $note_id,
            user_id = $user_id,
            emoji = $emoji,
            created_at = time::now()
    "#;

    state
        .surreal()
        .query(create_query)
        .bind(("note_id", note_id.to_string()))
        .bind(("user_id", auth_user.user_id.to_string()))
        .bind(("emoji", query.emoji.clone()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(to_status_response(&note, &state.config().instance_url)))
}

/// リアクションを外す
pub async fn unreact_note(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
    Query(query): Query<ReactQuery>,
) -> Result<Json<StatusResponse>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // 投稿の存在確認
    let note: Option<Note> = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    let note = note.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    // リアクションを削除
    let delete_query = r#"
        DELETE note_reaction WHERE note_id = $note_id AND user_id = $user_id AND emoji = $emoji
    "#;

    state
        .surreal()
        .query(delete_query)
        .bind(("note_id", note_id.to_string()))
        .bind(("user_id", auth_user.user_id.to_string()))
        .bind(("emoji", query.emoji.clone()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(to_status_response(&note, &state.config().instance_url)))
}

/// リアクション一覧を取得
pub async fn get_reactions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // 投稿の存在確認
    let note: Option<Note> = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    if note.is_none() {
        return Err(AppError::NotFound("Note not found".to_string()));
    }

    // リアクションを取得
    let reactions: Vec<serde_json::Value> = state
        .surreal()
        .query("SELECT * FROM note_reaction WHERE note_id = $note_id ORDER BY created_at DESC")
        .bind(("note_id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    Ok(Json(reactions))
}

/// 投稿状態を取得
pub async fn get_note_state(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // 投稿の存在確認
    let note: Option<Note> = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    if note.is_none() {
        return Err(AppError::NotFound("Note not found".to_string()));
    }

    // お気に入り状態確認
    let favorite_query = r#"
        SELECT * FROM favorite WHERE note_id = $note_id AND user_id = $user_id
    "#;
    let favorite_exists: Vec<crate::models::Favorite> = state
        .surreal()
        .query(favorite_query)
        .bind(("note_id", note_id.to_string()))
        .bind(("user_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    let is_favorited = !favorite_exists.is_empty();

    Ok(Json(serde_json::json!({
        "favorited": is_favorited
    })))
}

/// Note を StatusResponse に変換
fn to_status_response(note: &Note, instance_url: &str) -> StatusResponse {
    let visibility_str = match note.visibility {
        NoteVisibility::Public => "public",
        NoteVisibility::Home => "unlisted",
        NoteVisibility::Followers => "private",
        NoteVisibility::Specified => "direct",
    };

    StatusResponse {
        id: note.id.to_string(),
        uri: note.uri.clone().unwrap_or_else(|| {
            format!("{}/notes/{}", instance_url, note.id)
        }),
        url: note.uri.clone().or_else(|| {
            Some(format!("{}/notes/{}", instance_url, note.id))
        }),
        account_id: note.actor_id.to_string(),
        content: note.text.clone().unwrap_or_default(),
        created_at: note.created_at.to_rfc3339(),
        in_reply_to_id: note.reply_id.map(|id| id.to_string()),
        in_reply_to_account_id: note.reply_actor_id.map(|id| id.to_string()),
        reblog: None,
        sensitive: note.cw.is_some(),
        spoiler_text: note.cw.clone().unwrap_or_default(),
        visibility: visibility_str.to_string(),
        replies_count: note.replies_count,
        reblogs_count: note.renote_count,
        favourites_count: note.total_reactions(),
        favourited: false,
        reblogged: false,
        muted: false,
        bookmarked: false,
        pinned: false,
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: note.tags.iter().map(|tag| {
            serde_json::json!({
                "name": tag,
                "url": format!("{}/tags/{}", instance_url, tag),
            })
        }).collect(),
        emojis: Vec::new(),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SearchByTagQuery {
    pub tag: String,
    pub limit: Option<usize>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
    pub reply: Option<bool>,
    pub renote: Option<bool>,
    pub with_files: Option<bool>,
    pub poll: Option<bool>,
}

/// タグベースのノート検索
pub async fn search_by_tag(
    auth_user: Option<AuthUser>,
    State(state): State<AppState>,
    Query(query): Query<SearchByTagQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let limit = query.limit.unwrap_or(10).min(100);
    let tag = query.tag.to_lowercase();

    let mut query_str = "SELECT * FROM note WHERE $tag INSIDE tags".to_string();
    query_str.push_str(" AND visibility IN ['public', 'home']");

    if query.reply == Some(false) { query_str.push_str(" AND reply_id IS NONE"); }
    if query.renote == Some(false) { query_str.push_str(" AND renote_id IS NONE"); }
    if query.with_files == Some(true) { query_str.push_str(" AND file_ids != []"); }
    if query.poll == Some(true) { query_str.push_str(" AND poll IS NOT NONE"); }
    if let Some(since) = &query.since_id { query_str.push_str(" AND id > $since_id"); }
    if let Some(until) = &query.until_id { query_str.push_str(" AND id < $until_id"); }

    query_str.push_str(" ORDER BY created_at DESC LIMIT $limit");

    let mut surreal_query = state.surreal()
        .query(&query_str)
        .bind(("tag", tag))
        .bind(("limit", limit));
    if let Some(s) = &query.since_id { surreal_query = surreal_query.bind(("since_id", s.clone())); }
    if let Some(u) = &query.until_id { surreal_query = surreal_query.bind(("until_id", u.clone())); }

    let notes: Vec<Note> = surreal_query
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    let instance_url = state.config().instance_url.clone();
    Ok(Json(notes.iter().map(|n| to_status_response(n, &instance_url)).collect()))
}

/// リノート解除
pub async fn unrenote(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>> {
    let note_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // Find the user's renote of this note
    let renote: Option<Note> = state.surreal()
        .query("SELECT * FROM note WHERE actor_id = $user_id AND renote_id = $note_id AND text IS NONE LIMIT 1")
        .bind(("user_id", auth_user.user_id.to_string()))
        .bind(("note_id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    let renote = renote.ok_or_else(|| AppError::NotFound("Renote not found".to_string()))?;

    // Delete the renote
    state.surreal()
        .query("DELETE note WHERE id = $id")
        .bind(("id", renote.id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    // Decrement renote count on original note
    state.surreal()
        .query("UPDATE note SET renote_count = renote_count - 1 WHERE id = $id AND renote_count > 0")
        .bind(("id", note_id.to_string()))
        .await
        .ok();

    // Decrement user note count
    state.surreal()
        .query("UPDATE user SET notes_count = notes_count - 1 WHERE id = $id AND notes_count > 0")
        .bind(("id", auth_user.user_id.to_string()))
        .await
        .ok();

    // Return the original note
    let original: Option<Note> = state.surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    let original = original.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    Ok(Json(to_status_response(&original, &state.config().instance_url)))
}
