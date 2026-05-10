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
pub struct NoteQuery {
    pub formatted: Option<bool>,
}

/// ノートをActivityPubオブジェクトとして返却
pub async fn get_note(
    State(state): State<AppState>,
    Path(note_id): Path<String>,
    Query(_query): Query<NoteQuery>,
) -> Result<Json<serde_json::Value>> {
    let note_ulid = note_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation(format!("Invalid note ID: {}", note_id)))?;

    // ノート取得
    let mut result = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id")
        .bind(("id", note_ulid.to_string()))
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            AppError::Database(e)
        })?;

    let note: Option<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize note: {}", e))
    })?;

    let note = note.ok_or_else(|| AppError::NotFound(crate::t!("note-not-found")))?;

    // 公開範囲チェック（非公開ノートは返さない）
    match note.visibility {
        NoteVisibility::Public | NoteVisibility::Home => (),
        _ => return Err(AppError::Forbidden(crate::t!("error-forbidden"))),
    }

    // リモートノートの場合はリダイレクトすべきだが、ここではエラー
    if note.host.is_some() {
        return Err(AppError::NotFound(crate::t!("note-not-found")));
    }

    let instance_url = &state.config().instance_url;
    let note_url = format!("{}/notes/{}", instance_url, note.id);

    // ActivityPub Noteオブジェクトを構築
    let ap_note = build_ap_note(&note, instance_url);

    Ok(Json(ap_note))
}

/// NoteをActivityPubオブジェクトに変換
fn build_ap_note(note: &Note, instance_url: &str) -> serde_json::Value {
    let note_url = format!("{}/notes/{}", instance_url, note.id);
    let actor_url = format!("{}/users/{}", instance_url, note.actor_id);

    // 公開範囲に応じたto/ccを設定
    let (to, cc) = match note.visibility {
        NoteVisibility::Public => {
            (vec!["https://www.w3.org/ns/activitystreams#Public".to_string()], vec![])
        }
        NoteVisibility::Home => {
            // Unlisted - PublicとFollowers
            (vec!["https://www.w3.org/ns/activitystreams#Public".to_string()], vec![])
        }
        _ => {
            // その他は直接指定
            (vec![], vec![])
        }
    };

    let mut obj = serde_json::json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            {
                "ostatus": "http://ostatus.org#",
                "atomUri": "ostatus:atomUri",
                "inReplyToAtomUri": "ostatus:inReplyToAtomUri",
                "conversation": "ostatus:conversation",
                "sensitive": "as:sensitive",
                "Hashtag": "as:Hashtag",
                "Emoji": "toot:Emoji",
                "toot": "http://joinmastodon.org/ns#",
                "Misskey": "https://misskey-hub.net/ns#",
                "_misskey_content": "Misskey:_misskey_content",
                "_misskey_quote": "Misskey:_misskey_quote",
                "_misskey_reaction": "Misskey:_misskey_reaction",
                "_misskey_votes": "Misskey:_misskey_votes",
                "_misskey_talk": "Misskey:_misskey_talk",
            }
        ],
        "id": note_url,
        "type": "Note",
        "attributedTo": actor_url,
        "content": note.text.clone().unwrap_or_default(),
        "published": note.created_at.to_rfc3339(),
        "url": note_url,
        "attributedTo": actor_url,
        "to": to,
        "cc": cc,
        "sensitive": note.cw.is_some(),
    });

    // リプライ情報
    if let Some(reply_id) = note.reply_id {
        let reply_url = format!("{}/notes/{}", instance_url, reply_id);
        obj["inReplyTo"] = serde_json::json!(reply_url);
    }

    // CW/サマリー
    if let Some(cw) = &note.cw {
        obj["summary"] = serde_json::json!(cw);
    }

    // ハッシュタグ
    if !note.tags.is_empty() {
        let tags: Vec<_> = note.tags.iter().map(|tag| {
            serde_json::json!({
                "type": "Hashtag",
                "href": format!("{}/tags/{}", instance_url, tag),
                "name": format!("#{}", tag),
            })
        }).collect();
        obj["tag"] = serde_json::json!(tags);
    }

    // リアクション（簡易実装）
    let reactions_count = note.total_reactions();
    if reactions_count > 0 {
        let reactions = serde_json::json!({
            "type": "_misskey_reactions",
            "count": reactions_count,
        });
        obj["_misskey_reaction"] = reactions;
    }

    obj
}

/// ノートのアクティビティを返却（Create<Note>）
pub async fn get_note_activity(
    State(state): State<AppState>,
    Path(note_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let note_ulid = note_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // ノート取得
    let mut result = state
        .surreal()
        .query("SELECT * FROM note WHERE id = $id AND host IS NULL")
        .bind(("id", note_ulid.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let note: Option<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize: {}", e))
    })?;

    let note = note.ok_or_else(|| AppError::NotFound(crate::t!("note-not-found")))?;

    // 公開範囲チェック
    match note.visibility {
        NoteVisibility::Public | NoteVisibility::Home => (),
        _ => return Err(AppError::Forbidden(crate::t!("error-forbidden"))),
    }

    let instance_url = &state.config().instance_url;
    let actor_url = format!("{}/users/{}", instance_url, note.actor_id);
    let note_url = format!("{}/notes/{}", instance_url, note.id);

    // Createアクティビティを構築
    let ap_note = build_ap_note(&note, instance_url);
    let activity = serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{}/activity", note_url),
        "type": "Create",
        "actor": actor_url,
        "published": note.created_at.to_rfc3339(),
        "object": ap_note,
    });

    Ok(Json(activity))
}
