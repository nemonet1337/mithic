use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use tracing::error;
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{Note, NoteVisibility},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct OutboxQuery {
    pub page: Option<bool>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

/// Outboxエンドポイント - ActivityPub仕様準拠
pub async fn outbox(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<OutboxQuery>,
) -> Result<Json<serde_json::Value>> {
    let instance_url = &state.config().instance_url;
    let actor_url = format!("{}/users/{}", instance_url, username);
    let outbox_url = format!("{}/outbox", actor_url);

    // ユーザー存在確認と投稿数取得
    let mut result = state
        .surreal()
        .query("SELECT id, notes_count FROM user WHERE username_lower = $username")
        .bind(("username", username.to_lowercase()))
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            AppError::Database(e)
        })?;

    let user_data: Option<(String, i32)> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize: {}", e))
    })?;

    let (user_id, notes_count) = user_data
        .ok_or_else(|| AppError::NotFound(crate::t!("actor-not-found")))?;

    let page = query.page.unwrap_or(false);

    if page {
        // Pageリクエスト - 実際のアクティビティ一覧
        let limit = 20;

        // ページングパラメータ
        let since_id = query.since_id.as_ref().and_then(|s| s.parse::<Ulid>().ok());
        let until_id = query.until_id.as_ref().and_then(|u| u.parse::<Ulid>().ok());

        // ノート取得クエリ構築
        let mut note_query = String::from(
            "SELECT * FROM note WHERE actor_id = user:$user_id AND visibility IN ['public', 'home']"
        );

        if let Some(since) = &since_id {
            note_query.push_str(&format!(" AND id > '{}'", since));
        }

        if let Some(until) = &until_id {
            note_query.push_str(&format!(" AND id < '{}'", until));
        }

        note_query.push_str(" ORDER BY id DESC LIMIT $limit");

        let mut result = state
            .surreal()
            .query(&note_query)
            .bind(("user_id", user_id.clone()))
            .bind(("limit", limit + 1))
            .await
            .map_err(|e| AppError::Database(e))?;

        let notes: Vec<Note> = result.take(0).unwrap_or_default();

        let has_more = notes.len() > limit;
        let notes: Vec<_> = notes.into_iter().take(limit).collect();

        // アクティビティに変換
        let mut ordered_items = Vec::new();
        for note in notes {
            let activity = note_to_activity(&note, &actor_url, instance_url);
            ordered_items.push(activity);
        }

        // next/prev URL生成
        let next = if has_more && !notes.is_empty() {
            let last_id = notes.last().unwrap().id.to_string();
            Some(format!("{}?page=true&until_id={}", outbox_url, last_id))
        } else {
            None
        };

        let prev = if !notes.is_empty() {
            let first_id = notes.first().unwrap().id.to_string();
            Some(format!("{}?page=true&since_id={}", outbox_url, first_id))
        } else {
            None
        };

        // since_idが指定された場合は逆順なので調整
        let ordered_items = if since_id.is_some() {
            ordered_items.into_iter().rev().collect()
        } else {
            ordered_items
        };

        Ok(Json(serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}?page=true", outbox_url),
            "type": "OrderedCollectionPage",
            "partOf": outbox_url,
            "totalItems": notes_count,
            "orderedItems": ordered_items,
            "next": next,
            "prev": prev,
        })))
    } else {
        // Collectionリクエスト - メタ情報のみ
        Ok(Json(serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": outbox_url,
            "type": "OrderedCollection",
            "totalItems": notes_count,
            "first": format!("{}?page=true", outbox_url),
            "last": format!("{}?page=true&since_id=00000000000000000000000000", outbox_url),
        })))
    }
}

/// NoteをActivityPubアクティビティに変換
fn note_to_activity(note: &Note, actor_url: &str, instance_url: &str) -> serde_json::Value {
    let note_id = format!("{}/notes/{}", instance_url, note.id);

    // Noteオブジェクト
    let note_object = serde_json::json!({
        "id": note_id,
        "type": "Note",
        "attributedTo": actor_url,
        "content": note.text.clone().unwrap_or_default(),
        "published": note.created_at.to_rfc3339(),
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": [],
    });

    // Createアクティビティ
    serde_json::json!({
        "id": format!("{}/activity", note_id),
        "type": "Create",
        "actor": actor_url,
        "published": note.created_at.to_rfc3339(),
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": [],
        "object": note_object,
    })
}
