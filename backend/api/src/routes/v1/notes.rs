//! Notes CRUD / reactions / renote / favorite / pin / polls / search

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use mithic_core::models::note::{Note, NoteId, NoteVisibility};
use mithic_core::models::notification::{Notification, NotificationType};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::cache;
use mithic_db::queries::{
    add_favorite, add_reaction, create_note, delete_note, get_actor_by_id, get_note_by_id,
    get_note_quotes, get_note_replies, get_reaction_by_actor, is_following,
    remove_all_reactions_by_actor, remove_favorite, remove_reaction,
};
use serde::Deserialize;
use serde_json::Value;
use shared::{CreateNoteRequest, Note as NoteDto, ReactionSummary};

use crate::dto::{
    actor_to_user, apply_viewer_reaction, note_to_dto_full, reaction_summaries_from_map,
};
use crate::events::StreamBroadcast;
use crate::http_cache::{CC_PUBLIC_NOTE, json_with_cache};
use crate::routes::v1::common::{ok_null, parse_note_id};
use crate::services::note::{create_note_service, publish_notification};
use crate::state::AppState;

const NOTE_CACHE_TTL: u64 = 3600;

async fn ensure_note_visible(
    state: &AppState,
    note: &Note,
    viewer: Option<mithic_core::models::actor::ActorId>,
) -> Result<()> {
    match note.visibility {
        NoteVisibility::Public | NoteVisibility::Home => Ok(()),
        NoteVisibility::Followers => {
            let Some(viewer_id) = viewer else {
                return Err(AppError::Forbidden("Followers-only note".to_string()));
            };
            if viewer_id == note.actor_id {
                return Ok(());
            }
            let follows = is_following(state.surreal(), &viewer_id, &note.actor_id)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            if follows {
                Ok(())
            } else {
                Err(AppError::Forbidden("Followers-only note".to_string()))
            }
        }
        NoteVisibility::Specified => {
            let Some(viewer_id) = viewer else {
                return Err(AppError::Forbidden("Specified-visibility note".to_string()));
            };
            if viewer_id == note.actor_id || note.visible_user_ids.contains(&viewer_id) {
                Ok(())
            } else {
                Err(AppError::Forbidden("Specified-visibility note".to_string()))
            }
        }
    }
}

pub async fn fetch_note_dto(state: &AppState, note_id: &NoteId) -> Result<(Note, NoteDto)> {
    let note = get_note_by_id(state.surreal(), note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    let author = get_actor_by_id(state.surreal(), &note.actor_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;
    let dto = note_to_dto_full(state, &note, actor_to_user(&author)).await;
    Ok((note, dto))
}

pub async fn create_note_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<CreateNoteRequest>,
) -> Result<Json<NoteDto>> {
    let dto = create_note_service(&state, auth.user_id, request).await?;
    let note_key = format!("note:{}", dto.id);
    let _ = cache::set_json(state.dragonfly(), &note_key, &dto, NOTE_CACHE_TTL).await;
    Ok(Json(dto))
}

pub async fn show_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    auth: Option<Extension<AuthUser>>,
    Path(id): Path<String>,
) -> Result<Response> {
    let note_id = parse_note_id(&id)?;
    let viewer = auth.map(|Extension(a)| a.user_id);

    let note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    ensure_note_visible(&state, &note, viewer).await?;

    let is_publicish = matches!(
        note.visibility,
        NoteVisibility::Public | NoteVisibility::Home
    );

    if is_publicish {
        let note_key = format!("note:{note_id}");
        if let Some(mut dto) = cache::get_json::<NoteDto>(state.dragonfly(), &note_key).await {
            if let Some(vid) = viewer {
                apply_viewer_reaction(&state, &mut dto, &vid.to_string()).await;
                return Ok(Json(dto).into_response());
            }
            return Ok(json_with_cache(&headers, dto, CC_PUBLIC_NOTE));
        }
    }

    let author = get_actor_by_id(state.surreal(), &note.actor_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    let mut dto = note_to_dto_full(&state, &note, actor_to_user(&author)).await;
    if is_publicish {
        let note_key = format!("note:{note_id}");
        let _ = cache::set_json(state.dragonfly(), &note_key, &dto, NOTE_CACHE_TTL).await;
        if let Some(vid) = viewer {
            apply_viewer_reaction(&state, &mut dto, &vid.to_string()).await;
            return Ok(Json(dto).into_response());
        }
        return Ok(json_with_cache(&headers, dto, CC_PUBLIC_NOTE));
    }
    if let Some(vid) = viewer {
        apply_viewer_reaction(&state, &mut dto, &vid.to_string()).await;
    }
    Ok(Json(dto).into_response())
}

pub async fn delete_note_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    let note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    if note.actor_id != auth.user_id {
        return Err(AppError::Forbidden(
            "You can only delete your own notes".to_string(),
        ));
    }
    delete_note(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::delete(state.dragonfly(), &format!("note:{note_id}")).await;
    state.publish_stream(StreamBroadcast::NoteDeleted {
        id: note_id.to_string(),
    });
    let _ = state
        .surreal()
        .query(
            "UPDATE user SET notes_count = <int>(notes_count OR 1) - 1 WHERE id = type::record('user', $id);",
        )
        .bind(("id", auth.user_id.to_string()))
        .await;
    Ok(ok_null())
}

pub async fn note_replies(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<NoteDto>>> {
    let note_id = parse_note_id(&id)?;
    let rows = get_note_replies(state.surreal(), &note_id, 100)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(
        crate::routes::v1::common::rows_to_dtos(&state, rows).await,
    ))
}

pub async fn note_quotes(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<NoteDto>>> {
    let note_id = parse_note_id(&id)?;
    let rows = get_note_quotes(state.surreal(), &note_id, 100)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(
        crate::routes::v1::common::rows_to_dtos(&state, rows).await,
    ))
}

#[derive(Debug, Deserialize)]
pub struct ReactionBody {
    /// フロントは `emoji`、旧クライアント互換で `reaction` も受理
    #[serde(alias = "reaction")]
    pub emoji: String,
}

pub async fn add_reaction_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(body): Json<ReactionBody>,
) -> Result<Json<Vec<ReactionSummary>>> {
    let note_id = parse_note_id(&id)?;
    let (note, dto) = fetch_note_dto(&state, &note_id).await?;
    let emoji = body.emoji.trim().to_string();
    if emoji.is_empty() {
        return Err(AppError::Validation("Reaction is required".to_string()));
    }
    if let Ok(Some(author)) = get_actor_by_id(state.surreal(), &note.actor_id).await {
        if author.reaction_acceptance.as_deref() == Some("likeOnly")
            && emoji.starts_with(':')
            && emoji.ends_with(':')
            && emoji.len() > 2
        {
            return Err(AppError::Validation(
                "This account only accepts likes".to_string(),
            ));
        }
    }

    let existing = get_reaction_by_actor(
        state.surreal(),
        &note_id.to_string(),
        &auth.user_id.to_string(),
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let sender = get_actor_by_id(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut added = false;
    match existing {
        Some(prev) if prev == emoji => {
            remove_reaction(
                state.surreal(),
                &note_id.to_string(),
                &auth.user_id.to_string(),
                &emoji,
            )
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
            if let Some(ref reactor) = sender {
                crate::services::note::deliver_reaction(&state, reactor, &note, &emoji, true).await;
            }
        }
        Some(prev) => {
            let _ = remove_all_reactions_by_actor(
                state.surreal(),
                &note_id.to_string(),
                &auth.user_id.to_string(),
            )
            .await;
            if let Some(ref reactor) = sender {
                crate::services::note::deliver_reaction(&state, reactor, &note, &prev, true).await;
            }
            added = true;
        }
        None => added = true,
    }

    if added {
        add_reaction(
            state.surreal(),
            &note_id.to_string(),
            &auth.user_id.to_string(),
            &emoji,
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if note.actor_id != auth.user_id {
            let notif = Notification::reaction(note.actor_id, auth.user_id, note_id, emoji.clone());
            publish_notification(&state, &notif, sender.as_ref(), Some(dto)).await;
        }

        if let Some(ref reactor) = sender {
            crate::services::note::deliver_reaction(&state, reactor, &note, &emoji, false).await;
        }
    }

    let _ = cache::delete(state.dragonfly(), &format!("note:{note_id}")).await;
    let mine = get_reaction_by_actor(
        state.surreal(),
        &note_id.to_string(),
        &auth.user_id.to_string(),
    )
    .await
    .ok()
    .flatten();
    let note = get_note_by_id(state.surreal(), &note_id)
        .await
        .ok()
        .flatten()
        .unwrap_or(note);
    Ok(Json(reaction_summaries_from_map(
        &note.reactions,
        mine.as_deref(),
    )))
}

pub async fn remove_reaction_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((id, emoji)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    let note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    remove_reaction(
        state.surreal(),
        &note_id.to_string(),
        &auth.user_id.to_string(),
        &emoji,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Ok(Some(reactor)) = get_actor_by_id(state.surreal(), &auth.user_id).await {
        crate::services::note::deliver_reaction(&state, &reactor, &note, &emoji, true).await;
    }

    let _ = cache::delete(state.dragonfly(), &format!("note:{note_id}")).await;
    Ok(ok_null())
}

pub async fn renote_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<NoteDto>> {
    let target_note_id = parse_note_id(&id)?;
    let (target_note, target_dto) = fetch_note_dto(&state, &target_note_id).await?;
    ensure_note_visible(&state, &target_note, Some(auth.user_id)).await?;
    if !matches!(
        target_note.visibility,
        NoteVisibility::Public | NoteVisibility::Home
    ) {
        return Err(AppError::Forbidden(
            "Cannot renote a non-public note".to_string(),
        ));
    }

    let mut renote = Note::new(auth.user_id, None, NoteVisibility::Public);
    renote.renote_id = Some(target_note_id);

    let created = create_note(state.surreal(), &renote)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    state
        .surreal()
        .query("UPDATE note SET renote_count += 1 WHERE id = type::record('note', $id);")
        .bind(("id", target_note_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let author = get_actor_by_id(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    if target_note.actor_id != auth.user_id {
        let notif = Notification::new(
            NotificationType::Renote,
            target_note.actor_id,
            Some(auth.user_id),
            Some(created.id),
        );
        publish_notification(&state, &notif, Some(&author), Some(target_dto)).await;
    }

    let dto = note_to_dto_full(&state, &created, actor_to_user(&author)).await;
    state.publish_stream(StreamBroadcast::Note(Box::new(dto.clone())));
    Ok(Json(dto))
}

pub async fn unrenote_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let target_note_id = parse_note_id(&id)?;
    let mut del = state
        .surreal()
        .query(
            "
        DELETE note WHERE actor_id = type::record('user', $my_id)
          AND renote_id = type::record('note', $target_id) RETURN BEFORE;
        ",
        )
        .bind(("my_id", auth.user_id.to_string()))
        .bind(("target_id", target_note_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let deleted_rows: Vec<surrealdb::types::Value> =
        del.take(0).map_err(|e| AppError::Internal(e.to_string()))?;
    let deleted_count = deleted_rows.len() as i64;

    if deleted_count > 0 {
        state
            .surreal()
            .query(
                "UPDATE note SET renote_count = math::max(renote_count - $n, 0) WHERE id = type::record('note', $id);",
            )
            .bind(("id", target_note_id.to_string()))
            .bind(("n", deleted_count))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(ok_null())
}

pub async fn favorite_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    let _ = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    add_favorite(state.surreal(), &auth.user_id, &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

pub async fn unfavorite_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    remove_favorite(state.surreal(), &auth.user_id, &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

pub async fn pin_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    let _ = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    let mut count_res = state
        .surreal()
        .query(
            "SELECT count() AS c FROM user_note_pining WHERE in = type::record('user', $user) GROUP ALL;",
        )
        .bind(("user", auth.user_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    #[derive(serde::Deserialize)]
    struct CountRow {
        c: i64,
    }
    let rows: Vec<CountRow> = {
        let raw: Vec<surrealdb::types::Value> = count_res
            .take(0)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        mithic_db::queries::rows_to(raw).map_err(|e| AppError::Internal(e.to_string()))?
    };
    let pinned = rows.first().map(|r| r.c).unwrap_or(0);
    if pinned >= 5 {
        return Err(AppError::Validation(
            "Maximum of 5 pinned notes".to_string(),
        ));
    }

    state
        .surreal()
        .query(
            "
        RELATE (type::record('user', $user)) -> user_note_pining -> (type::record('note', $note));
        ",
        )
        .bind(("user", auth.user_id.to_string()))
        .bind(("note", note_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

pub async fn unpin_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    let _ = state
        .surreal()
        .query(
            "
        DELETE user_note_pining WHERE in = type::record('user', $user) AND out = type::record('note', $note);
        ",
        )
        .bind(("user", auth.user_id.to_string()))
        .bind(("note", note_id.to_string()))
        .await;
    Ok(ok_null())
}

#[derive(Debug, Deserialize)]
pub struct VoteBody {
    pub choice: usize,
}

pub async fn vote_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    let note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    if !note.has_poll {
        return Err(AppError::Validation("Note is not a poll".to_string()));
    }

    mithic_db::queries::vote_poll(
        state.surreal(),
        &note_id.to_string(),
        &auth.user_id.to_string(),
        body.choice,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

#[derive(Debug, Deserialize)]
pub struct SearchNotesQuery {
    pub q: String,
    pub limit: Option<usize>,
}

pub async fn search_notes(
    State(state): State<AppState>,
    Query(query): Query<SearchNotesQuery>,
) -> Result<Json<Vec<NoteDto>>> {
    let limit = query.limit.unwrap_or(20).min(100);
    let mut response = state
        .surreal()
        .query(
            "
            SELECT
                *,
                actor_id.id AS actor_id,
                reply_id.id AS reply_id,
                renote_id.id AS renote_id,
                actor_id.* AS author
            FROM note
            WHERE text CONTAINS $query
              AND (visibility = 'public' OR visibility = 'home')
            ORDER BY id DESC
            LIMIT $limit;
            ",
        )
        .bind(("query", query.q))
        .bind(("limit", limit as i64))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<surrealdb::types::Value> = response
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let notes: Vec<mithic_db::queries::NoteWithAuthor> =
        mithic_db::queries::rows_to(rows).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(
        crate::routes::v1::common::rows_to_dtos(&state, notes).await,
    ))
}
