//! ノート作成サービス
//!
//! 投稿作成を縦串で処理する:
//! DB 保存 → カウンタ更新 → 通知生成 → ストリーム配信 → ActivityPub 配送キュー投入

use mithic_core::misc::extract_hashtags::extract_hashtags;
use mithic_core::misc::extract_mentions::extract_local_mentions;
use mithic_core::models::actor::{Actor, ActorId};
use mithic_core::models::note::{Note, NoteVisibility};
use mithic_core::models::notification::{Notification, NotificationType};
use mithic_core::{AppError, Result};
use mithic_db::queries::{
    count_followers, create_note, create_notification, get_actor_by_id, get_note_by_id,
};
use shared::{CreateNoteRequest, Note as NoteDto, Notification as NotifDto};

use crate::dto::{actor_to_user, note_to_dto_full, notif_type_to_dto, visibility_from_dto};
use crate::events::StreamBroadcast;
use crate::routes::v1::common::parse_note_id;
use crate::state::AppState;

/// ノート本文の最大文字数
const MAX_NOTE_CHARS: usize = 3000;

/// 投稿を作成し、関連する副作用 (通知・ストリーム・連合) を処理する
pub async fn create_note_service(
    state: &AppState,
    author_id: ActorId,
    request: CreateNoteRequest,
) -> Result<NoteDto> {
    if request.text.chars().count() > MAX_NOTE_CHARS {
        return Err(AppError::Validation(format!(
            "Note text must be at most {MAX_NOTE_CHARS} characters"
        )));
    }

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

    // 投稿数カウンタ更新 (レスポンスに不要 → バックグラウンド)
    {
        let surreal = state.surreal().clone();
        let id = author_id.to_string();
        tokio::spawn(async move {
            let _ = surreal
                .query("UPDATE user SET notes_count += 1 WHERE id = type::record('user', $id);")
                .bind(("id", id))
                .await;
        });
    }

    let dto = note_to_dto_full(state, &created, actor_to_user(&author)).await;

    // 通知生成はレスポンスに不要 → バックグラウンド (メンションは一括解決)
    {
        let state = state.clone();
        let author = author.clone();
        let dto = dto.clone();
        let created = created.clone();
        tokio::spawn(async move {
            spawn_note_notifications(state, author, created, dto).await;
        });
    }

    // ストリーム配信 (specified/followers はタイムラインへ流さない)
    if matches!(
        created.visibility,
        NoteVisibility::Public | NoteVisibility::Home
    ) {
        state.publish_stream(StreamBroadcast::Note(Box::new(dto.clone())));
    }

    // タイムラインへの fan-out (Push モデル、ハイブリッド閾値 10,000)
    if created.visibility == NoteVisibility::Public {
        let score = created.created_at.timestamp_millis() as f64;
        let author_id_for_fanout = author_id;
        let created_id = created.id;
        let dragonfly = state.dragonfly().clone();
        let surreal = state.surreal().clone();

        tokio::spawn(async move {
            let follower_count = match count_followers(&surreal, &author_id_for_fanout).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Failed to count followers for fan-out: {}", e);
                    return;
                }
            };

            const FANOUT_THRESHOLD: usize = 10_000;
            if follower_count < FANOUT_THRESHOLD {
                let followers = match mithic_db::queries::get_followers(
                    &surreal,
                    &author_id_for_fanout,
                )
                .await
                {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::warn!("Failed to get followers for fan-out: {}", e);
                        return;
                    }
                };

                for follower in followers {
                    let key = format!("home_timeline:{}", follower.id);
                    let _ = mithic_db::cache::timeline_push(
                        &dragonfly,
                        &key,
                        &created_id.to_string(),
                        score,
                    )
                    .await;
                }
                let _ = mithic_db::cache::timeline_push(
                    &dragonfly,
                    "local_timeline",
                    &created_id.to_string(),
                    score,
                )
                .await;
                let _ = mithic_db::cache::timeline_push(
                    &dragonfly,
                    "global_timeline",
                    &created_id.to_string(),
                    score,
                )
                .await;
                // 公開 TL の JSON キャッシュを無効化
                mithic_db::cache::invalidate_public_timelines(&dragonfly).await;
            } else {
                // fan-out スキップ時も JSON キャッシュは消す
                mithic_db::cache::invalidate_public_timelines(&dragonfly).await;
                tracing::debug!(
                    "Follower count {} >= threshold {}, skipping dragonfly fan-out (Pull model)",
                    follower_count,
                    FANOUT_THRESHOLD
                );
            }
        });
    }

    // ActivityPub 配送 (public のみ)
    if created.visibility == NoteVisibility::Public && author.private_key.is_some() {
        let activity =
            build_create_activity(state.config().instance_url.as_str(), &author, &created);
        let federation = state.federation_service().clone();
        let actor_id_str = author_id.to_string();
        let author_clone = author.clone();
        tokio::spawn(async move {
            // フォロワーへの配送
            if let Err(e) = federation
                .broadcast_to_followers(activity.clone(), &actor_id_str, &author_clone)
                .await
            {
                tracing::warn!("Failed to queue federation delivery: {}", e);
            }
            // リレーへの配送
            federation.fanout_to_relays(&activity).await.ok();
        });
    }

    Ok(dto)
}

/// 返信・メンション通知をバックグラウンドで生成する
async fn spawn_note_notifications(state: AppState, author: Actor, created: Note, dto: NoteDto) {
    // 返信通知
    if let Some(reply_id) = created.reply_id {
        if let Ok(Some(parent)) = get_note_by_id(state.surreal(), &reply_id).await {
            if parent.actor_id != author.id {
                let notif = Notification::new(
                    NotificationType::Reply,
                    parent.actor_id,
                    Some(author.id),
                    Some(created.id),
                );
                publish_notification(&state, &notif, Some(&author), Some(dto.clone())).await;
            }
        }
    }

    // メンション通知: ユニーク username を一括解決
    if let Some(ref text) = created.text {
        let mentions = extract_local_mentions(text);
        let mut names: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for m in &mentions {
            let lower = m.username.to_lowercase();
            if seen.insert(lower) {
                names.push(m.username.clone());
            }
        }
        if !names.is_empty() {
            // 1 クエリで解決 (username_lower IN)
            let mut response = match state
                .surreal()
                .query("SELECT * FROM user WHERE username_lower IN $names AND host = NONE;")
                .bind((
                    "names",
                    names.iter().map(|n| n.to_lowercase()).collect::<Vec<_>>(),
                ))
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!("Failed to resolve mentions: {e}");
                    return;
                }
            };
            let rows: Vec<surrealdb::types::Value> = match response.take(0) {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!("Failed to take mention rows: {e}");
                    return;
                }
            };
            let actors: Vec<Actor> = match mithic_db::queries::rows_to(rows) {
                Ok(a) => a,
                Err(e) => {
                    tracing::warn!("Failed to deserialize mentioned actors: {e}");
                    return;
                }
            };
            for mentioned in actors {
                if mentioned.id != author.id {
                    let notif = Notification::new(
                        NotificationType::Mention,
                        mentioned.id,
                        Some(author.id),
                        Some(created.id),
                    );
                    publish_notification(&state, &notif, Some(&author), Some(dto.clone())).await;
                }
            }
        }
    }
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
        notification: Box::new(dto.clone()),
    });

    // Web Push (background; optional if VAPID configured)
    let state = state.clone();
    let recipient = notif.recipient_id;
    tokio::spawn(async move {
        crate::services::push::deliver_web_push(&state, recipient, &dto).await;
    });
}

/// ActivityPub JSON-LD コンテキスト (Misskey 拡張含む)
fn ap_context() -> serde_json::Value {
    serde_json::json!([
        "https://www.w3.org/ns/activitystreams",
        {
            "misskey": "https://misskey-hub.net/ns#",
            "_misskey_reaction": "misskey:_misskey_reaction",
            "_misskey_quote": "misskey:_misskey_quote",
            "quoteUrl": "http://fedibird.com/ns#quoteUrl",
            "quoteUri": "http://fedibird.com/ns#quoteUri",
            "sensitive": "as:sensitive",
            "toot": "http://joinmastodon.org/ns#",
            "Emoji": "toot:Emoji",
            "Hashtag": "as:Hashtag"
        }
    ])
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
    let source_text = note.text.clone().unwrap_or_default();
    // AP content は HTML。comrak は生 HTML をエスケープするので XSS 面は安全
    let content_html = shared::markdown::render_markdown(&source_text);

    // 引用リノート: renote_id + 本文あり → quoteUrl / _misskey_quote / FEP-e232 quote を併記
    let mut object = serde_json::json!({
        "id": note_uri,
        "type": "Note",
        "attributedTo": actor_uri,
        "content": content_html,
        "source": {
            "content": source_text,
            "mediaType": "text/markdown",
        },
        "summary": note.cw,
        "published": published,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": [followers_uri],
    });
    if let Some(renote_id) = note.renote_id {
        let target = format!("{instance_url}/notes/{renote_id}");
        if note.text.as_ref().is_some_and(|t| !t.trim().is_empty()) {
            object["quoteUrl"] = serde_json::json!(target);
            object["_misskey_quote"] = serde_json::json!(target);
            object["quote"] = serde_json::json!(target);
        }
    }

    serde_json::json!({
        "@context": ap_context(),
        "id": format!("{note_uri}/activity"),
        "type": "Create",
        "actor": actor_uri,
        "published": published,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": [followers_uri],
        "object": object,
    })
}

/// リアクションを Misskey 互換 Like として構築する
pub fn build_like_activity(
    instance_url: &str,
    actor_uri: &str,
    note_uri: &str,
    reaction: &str,
    custom_emoji_url: Option<&str>,
) -> serde_json::Value {
    let activity_id = format!("{instance_url}/likes/{}", ulid::Ulid::new());
    let mut activity = serde_json::json!({
        "@context": ap_context(),
        "id": activity_id,
        "type": "Like",
        "actor": actor_uri,
        "object": note_uri,
        "content": reaction,
        "_misskey_reaction": reaction,
    });
    if let Some(url) = custom_emoji_url {
        let name = reaction.trim_matches(':');
        activity["tag"] = serde_json::json!([{
            "type": "Emoji",
            "name": format!(":{name}:"),
            "icon": {
                "type": "Image",
                "url": url,
            }
        }]);
    }
    activity
}

/// Undo(Like) アクティビティ
pub fn build_undo_like_activity(
    instance_url: &str,
    actor_uri: &str,
    note_uri: &str,
    reaction: &str,
) -> serde_json::Value {
    let like = build_like_activity(instance_url, actor_uri, note_uri, reaction, None);
    let like_id = like
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    serde_json::json!({
        "@context": ap_context(),
        "id": format!("{instance_url}/undo/{}", ulid::Ulid::new()),
        "type": "Undo",
        "actor": actor_uri,
        "object": {
            "id": like_id,
            "type": "Like",
            "actor": actor_uri,
            "object": note_uri,
            "content": reaction,
            "_misskey_reaction": reaction,
        }
    })
}

/// ノートの正規 AP URI
pub fn note_ap_uri(instance_url: &str, note: &Note) -> String {
    note.uri
        .clone()
        .unwrap_or_else(|| format!("{}/notes/{}", instance_url.trim_end_matches('/'), note.id))
}

/// リアクションを連合へ配送 (フォロワー + リモート著者 inbox)
pub async fn deliver_reaction(
    state: &AppState,
    reactor: &Actor,
    note: &Note,
    reaction: &str,
    undo: bool,
) {
    if reactor.private_key.is_none() {
        return;
    }
    let instance = state.config().instance_url.clone();
    let actor_uri = reactor
        .uri
        .clone()
        .unwrap_or_else(|| reactor.actor_uri(&instance));
    let note_uri = note_ap_uri(&instance, note);

    let custom_url = if reaction.starts_with(':') && reaction.ends_with(':') {
        let name = reaction.trim_matches(':');
        let mut res = state
            .surreal()
            .query("SELECT url FROM emoji WHERE name = $name AND is_public = true LIMIT 1;")
            .bind(("name", name.to_string()))
            .await
            .ok();
        res.as_mut()
            .and_then(|r| r.take::<Vec<surrealdb::types::Value>>(0).ok())
            .and_then(|rows| rows.into_iter().next())
            .and_then(|v| {
                let j = v.into_json_value();
                j.get("url").and_then(|u| u.as_str()).map(String::from)
            })
    } else {
        None
    };

    let activity = if undo {
        build_undo_like_activity(&instance, &actor_uri, &note_uri, reaction)
    } else {
        build_like_activity(
            &instance,
            &actor_uri,
            &note_uri,
            reaction,
            custom_url.as_deref(),
        )
    };

    let federation = state.federation_service().clone();
    let reactor = reactor.clone();
    let note_author_id = note.actor_id;
    let surreal = state.surreal().clone();

    tokio::spawn(async move {
        // リモート著者の inbox へ直接
        if let Ok(Some(author)) = get_actor_by_id(&surreal, &note_author_id).await {
            if author.host.is_some() {
                if let Some(inbox) = author.shared_inbox.or(author.inbox) {
                    if let Err(e) = federation
                        .queue_delivery(activity.clone(), vec![inbox])
                        .await
                    {
                        tracing::warn!("Failed to queue reaction to author: {e}");
                    }
                }
            }
        }
        // フォロワー配信
        if let Err(e) = federation
            .broadcast_to_followers(activity, &reactor.id.to_string(), &reactor)
            .await
        {
            tracing::warn!("Failed to queue reaction broadcast: {e}");
        }
    });
}
