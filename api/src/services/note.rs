//! ノート作成サービス (TODO Phase 2 サービス層)
//!
//! 投稿作成を縦串で処理する:
//! DB 保存 → カウンタ更新 → 通知生成 → ストリーム配信 → ActivityPub 配送キュー投入

use mithic_core::misc::extract_hashtags::extract_hashtags;
use mithic_core::models::actor::{Actor, ActorId};
use mithic_core::models::note::{Note, NoteId, NoteVisibility};
use mithic_core::models::notification::{Notification, NotificationType};
use mithic_core::{AppError, Result};
use mithic_db::queries::{create_note, create_notification, get_actor_by_id, get_note_by_id};
use shared::{CreateNoteRequest, Note as NoteDto, Notification as NotifDto};

use crate::dto::{actor_to_user, note_to_dto, visibility_from_dto};
use crate::events::StreamBroadcast;
use crate::routes::notifications::notif_type_to_dto;
use crate::state::AppState;

fn parse_note_id(raw: &str) -> Result<NoteId> {
    raw.parse::<NoteId>()
        .map_err(|_| AppError::Validation("Invalid note id".to_string()))
}

/// 投稿を作成し、関連する副作用 (通知・ストリーム・連合) を処理する
pub async fn create_note_service(
    state: &AppState,
    author_id: ActorId,
    request: CreateNoteRequest,
) -> Result<NoteDto> {
    let text = if request.text.trim().is_empty() {
        None
    } else {
        Some(request.text.clone())
    };

    if text.is_none() && request.file_ids.is_empty() {
        return Err(AppError::Validation(
            "Note must have text or files".to_string(),
        ));
    }

    let mut note = Note::new(author_id, text, visibility_from_dto(request.visibility));
    note.cw = request.cw.clone();
    note.file_ids = request.file_ids.clone();
    if let Some(text) = &note.text {
        note.tags = extract_hashtags(text);
    }
    if let Some(reply_id) = &request.reply_id {
        note.reply_id = Some(parse_note_id(reply_id)?);
    }

    let created = create_note(state.surreal(), &note)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let author = get_actor_by_id(state.surreal(), &author_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    // 投稿数カウンタ更新
    let _ = state
        .surreal()
        .query("UPDATE user SET notes_count += 1 WHERE id = type::record('user', $id);")
        .bind(("id", author_id.to_string()))
        .await;

    let dto = note_to_dto(&created, actor_to_user(&author));

    // 返信通知
    if let Some(reply_id) = created.reply_id {
        if let Ok(Some(parent)) = get_note_by_id(state.surreal(), &reply_id).await {
            if parent.actor_id != author_id {
                let notif = Notification::new(
                    NotificationType::Reply,
                    parent.actor_id,
                    Some(author_id),
                    Some(created.id),
                );
                publish_notification(state, &notif, Some(&author), Some(dto.clone())).await;
            }
        }
    }

    // ストリーム配信 (specified/followers はタイムラインへ流さない)
    if matches!(
        created.visibility,
        NoteVisibility::Public | NoteVisibility::Home
    ) {
        state.publish_stream(StreamBroadcast::Note(Box::new(dto.clone())));
    }

    // ActivityPub 配送 (public のみ)
    if created.visibility == NoteVisibility::Public && author.private_key.is_some() {
        let activity =
            build_create_activity(state.config().instance_url.as_str(), &author, &created);
        let federation = state.federation_service().clone();
        let actor_id_str = author_id.to_string();
        let author_clone = author.clone();
        tokio::spawn(async move {
            if let Err(e) = federation
                .broadcast_to_followers(activity, &actor_id_str, &author_clone)
                .await
            {
                tracing::warn!("Failed to queue federation delivery: {}", e);
            }
        });
    }

    Ok(dto)
}

/// 通知を永続化し、ストリームへも配信する
pub async fn publish_notification(
    state: &AppState,
    notif: &Notification,
    sender: Option<&Actor>,
    note: Option<NoteDto>,
) {
    if let Err(e) = create_notification(state.surreal(), notif).await {
        tracing::warn!("Failed to persist notification: {}", e);
        return;
    }

    let dto = NotifDto {
        id: notif.id.to_string(),
        created_at: notif.created_at.to_rfc3339(),
        notification_type: notif_type_to_dto(notif.notification_type),
        sender: sender.map(actor_to_user),
        note,
        reaction: notif.reaction.clone(),
        is_read: notif.is_read,
    };

    state.publish_stream(StreamBroadcast::Notification {
        user_id: notif.recipient_id.to_string(),
        notification: Box::new(dto),
    });
}

/// ActivityPub Create アクティビティを構築する
fn build_create_activity(instance_url: &str, author: &Actor, note: &Note) -> serde_json::Value {
    let actor_uri = author
        .uri
        .clone()
        .unwrap_or_else(|| author.actor_uri(instance_url));
    let note_uri = format!("{}/notes/{}", instance_url, note.id);
    let followers_uri = format!("{actor_uri}/followers");
    let published = note.created_at.to_rfc3339();

    serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{note_uri}/activity"),
        "type": "Create",
        "actor": actor_uri,
        "published": published,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": [followers_uri],
        "object": {
            "id": note_uri,
            "type": "Note",
            "attributedTo": actor_uri,
            "content": note.text.clone().unwrap_or_default(),
            "summary": note.cw,
            "published": published,
            "to": ["https://www.w3.org/ns/activitystreams#Public"],
            "cc": [followers_uri],
        }
    })
}
