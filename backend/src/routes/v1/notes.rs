//! Notes CRUD / reactions / renote / favorite / pin / polls / search

use crate::db::cache;
use crate::db::queries::notes::{NOTE_COLUMNS, NoteRow};
use crate::db::queries::{
    add_favorite, add_reaction, create_note, delete_note, get_actor_by_id, get_note_by_id,
    get_note_quotes, get_note_replies, get_reaction_by_actor, is_following,
    remove_all_reactions_by_actor, remove_favorite, remove_reaction,
};
use crate::models::note::{Note, NoteId, NoteVisibility};
use crate::models::notification::{Notification, NotificationType};
use crate::{AppError, AuthUser, Result};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
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
    viewer: Option<crate::models::actor::ActorId>,
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
            let follows = is_following(state.db(), &viewer_id, &note.actor_id)
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
    let note = get_note_by_id(state.db(), note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    let author = get_actor_by_id(state.db(), &note.actor_id)
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
    let _ = cache::set_json(state.redis(), &note_key, &dto, NOTE_CACHE_TTL).await;
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

    let note = get_note_by_id(state.db(), &note_id)
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
        if let Some(mut dto) = cache::get_json::<NoteDto>(state.redis(), &note_key).await {
            if let Some(vid) = viewer {
                apply_viewer_reaction(&state, &mut dto, &vid.to_string()).await;
                return Ok(Json(dto).into_response());
            }
            return Ok(json_with_cache(&headers, dto, CC_PUBLIC_NOTE));
        }
    }

    let author = get_actor_by_id(state.db(), &note.actor_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    let mut dto = note_to_dto_full(&state, &note, actor_to_user(&author)).await;
    if is_publicish {
        let note_key = format!("note:{note_id}");
        let _ = cache::set_json(state.redis(), &note_key, &dto, NOTE_CACHE_TTL).await;
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
    let note = get_note_by_id(state.db(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    if note.actor_id != auth.user_id {
        return Err(AppError::Forbidden(
            "You can only delete your own notes".to_string(),
        ));
    }
    delete_note(state.db(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::delete(state.redis(), &format!("note:{note_id}")).await;
    state.publish_stream(StreamBroadcast::NoteDeleted {
        id: note_id.to_string(),
    });
    sqlx::query(
        "UPDATE users
         SET notes_count = GREATEST(notes_count - 1, 0)
         WHERE id = $1",
    )
    .bind(auth.user_id.to_string())
    .execute(state.db())
    .await
    .ok();
    Ok(ok_null())
}

pub async fn note_replies(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<NoteDto>>> {
    let note_id = parse_note_id(&id)?;
    let rows = get_note_replies(state.db(), &note_id, 100)
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
    let rows = get_note_quotes(state.db(), &note_id, 100)
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
    if let Ok(Some(author)) = get_actor_by_id(state.db(), &note.actor_id).await
        && author.reaction_acceptance.as_deref() == Some("likeOnly")
        && emoji.starts_with(':')
        && emoji.ends_with(':')
        && emoji.len() > 2
    {
        return Err(AppError::Validation(
            "This account only accepts likes".to_string(),
        ));
    }

    let existing =
        get_reaction_by_actor(state.db(), &note_id.to_string(), &auth.user_id.to_string())
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

    let sender = get_actor_by_id(state.db(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut added = false;
    match existing {
        Some(prev) if prev == emoji => {
            remove_reaction(
                state.db(),
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
                state.db(),
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
            state.db(),
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

    let _ = cache::delete(state.redis(), &format!("note:{note_id}")).await;
    let mine = get_reaction_by_actor(state.db(), &note_id.to_string(), &auth.user_id.to_string())
        .await
        .ok()
        .flatten();
    let note = get_note_by_id(state.db(), &note_id)
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
    let note = get_note_by_id(state.db(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    remove_reaction(
        state.db(),
        &note_id.to_string(),
        &auth.user_id.to_string(),
        &emoji,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Ok(Some(reactor)) = get_actor_by_id(state.db(), &auth.user_id).await {
        crate::services::note::deliver_reaction(&state, &reactor, &note, &emoji, true).await;
    }

    let _ = cache::delete(state.redis(), &format!("note:{note_id}")).await;
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

    let created = create_note(state.db(), &renote)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    sqlx::query("UPDATE notes SET renote_count = renote_count + 1 WHERE id = $1")
        .bind(target_note_id.to_string())
        .execute(state.db())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let author = get_actor_by_id(state.db(), &auth.user_id)
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
    let deleted: Vec<String> = sqlx::query_scalar(
        "DELETE FROM notes
         WHERE actor_id = $1 AND renote_id = $2
         RETURNING id",
    )
    .bind(auth.user_id.to_string())
    .bind(target_note_id.to_string())
    .fetch_all(state.db())
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let deleted_count = deleted.len() as i64;

    if deleted_count > 0 {
        sqlx::query(
            "UPDATE notes
             SET renote_count = GREATEST(renote_count - $2, 0)
             WHERE id = $1",
        )
        .bind(target_note_id.to_string())
        .bind(deleted_count)
        .execute(state.db())
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
    let _ = get_note_by_id(state.db(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    add_favorite(state.db(), &auth.user_id, &note_id)
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
    remove_favorite(state.db(), &auth.user_id, &note_id)
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
    let _ = get_note_by_id(state.db(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    let pinned: i64 = sqlx::query_scalar("SELECT count(*) FROM user_note_pins WHERE user_id = $1")
        .bind(auth.user_id.to_string())
        .fetch_one(state.db())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if pinned >= 5 {
        return Err(AppError::Validation(
            "Maximum of 5 pinned notes".to_string(),
        ));
    }

    sqlx::query(
        "INSERT INTO user_note_pins (user_id, note_id) VALUES ($1, $2)
         ON CONFLICT (user_id, note_id) DO NOTHING",
    )
    .bind(auth.user_id.to_string())
    .bind(note_id.to_string())
    .execute(state.db())
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
    sqlx::query("DELETE FROM user_note_pins WHERE user_id = $1 AND note_id = $2")
        .bind(auth.user_id.to_string())
        .bind(note_id.to_string())
        .execute(state.db())
        .await
        .ok();
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
    let note = get_note_by_id(state.db(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    if !note.has_poll {
        return Err(AppError::Validation("Note is not a poll".to_string()));
    }

    #[derive(sqlx::FromRow)]
    struct PollRow {
        id: String,
        choices: sqlx::types::Json<Vec<serde_json::Value>>,
    }

    let poll =
        sqlx::query_as::<_, PollRow>("SELECT id, choices FROM polls WHERE note_id = $1 LIMIT 1")
            .bind(note_id.to_string())
            .fetch_optional(state.db())
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::Validation("Note is not a poll".to_string()))?;

    let poll_id = poll.id;
    let n = poll.choices.0.len();
    if body.choice >= n {
        return Err(AppError::Validation("Choice out of range".to_string()));
    }

    let already: Option<String> = sqlx::query_scalar(
        "SELECT id FROM poll_votes WHERE poll_id = $1 AND actor_id = $2 LIMIT 1",
    )
    .bind(&poll_id)
    .bind(auth.user_id.to_string())
    .fetch_optional(state.db())
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    if already.is_some() {
        return Err(AppError::Validation("Already voted".to_string()));
    }

    crate::db::queries::vote_poll(state.db(), &poll_id, &auth.user_id.to_string(), body.choice)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

#[derive(Debug, Deserialize)]
pub struct SearchNotesQuery {
    pub q: String,
    pub limit: Option<usize>,
}

const SEARCH_VIS_PUBLIC_OR_HOME: &str = "(visibility = 'public' OR visibility = 'home')";
const SEARCH_VIS_HOME: &str = "visibility = 'home'";

fn cached_public_is_complete(page_len: usize, page_limit: usize) -> bool {
    page_len < page_limit
}

async fn query_search_notes(
    state: &AppState,
    q: &str,
    limit: usize,
    visibility: &str,
) -> Result<Vec<NoteDto>> {
    let sql = format!(
        "SELECT {NOTE_COLUMNS} FROM notes
         WHERE text ILIKE '%' || $1 || '%'
           AND {visibility}
         ORDER BY id DESC
         LIMIT $2"
    );
    let rows = sqlx::query_as::<_, NoteRow>(&sql)
        .bind(q)
        .bind(limit as i64)
        .fetch_all(state.db())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let notes: Vec<Note> = rows
        .into_iter()
        .map(|row| row.try_into())
        .collect::<std::result::Result<_, _>>()
        .map_err(|e: anyhow::Error| AppError::Internal(e.to_string()))?;

    let rows = crate::db::queries::timeline::attach_authors(state.db(), notes)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(crate::routes::v1::common::rows_to_dtos(state, rows).await)
}
/// 公開 TL がページに収まる間は Redis を先に見る。空の q は DB の ILIKE。
// ponytail: 公開TLがページに収まる間だけ Redis を正にする。ページが満杯なら SurrealDB。CONTAINS が遅くなったら SurrealDB FULLTEXT。Meilisearch/ES は足さない。
pub async fn search_notes(
    State(state): State<AppState>,
    Query(query): Query<SearchNotesQuery>,
) -> Result<Json<Vec<NoteDto>>> {
    let limit = query.limit.unwrap_or(20).min(100);
    let q = query.q;
    if q.is_empty() {
        return Ok(Json(
            query_search_notes(&state, &q, limit, SEARCH_VIS_PUBLIC_OR_HOME).await?,
        ));
    }

    for page_limit in [100usize, 50, 40, 20, 10] {
        let key = cache::timeline_json_key("global", page_limit);
        let Some(page) = cache::get_json::<Vec<NoteDto>>(state.redis(), &key).await else {
            continue;
        };
        // 大きい limit が満杯なら、より小さいキーは全件ではない。短いページを正にしない。
        if !cached_public_is_complete(page.len(), page_limit) {
            break;
        }
        let mut hits: Vec<NoteDto> = page
            .into_iter()
            .filter(|n| n.visibility == NoteVisibility::Public && n.content.contains(&q))
            .collect();
        hits.extend(query_search_notes(&state, &q, limit, SEARCH_VIS_HOME).await?);
        hits.sort_by(|a, b| b.id.cmp(&a.id));
        hits.dedup_by(|a, b| a.id == b.id);
        hits.truncate(limit);
        return Ok(Json(hits));
    }

    Ok(Json(
        query_search_notes(&state, &q, limit, SEARCH_VIS_PUBLIC_OR_HOME).await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::cached_public_is_complete;

    #[test]
    fn cached_public_is_complete_full_page_is_false() {
        assert!(!cached_public_is_complete(20, 20));
    }

    #[test]
    fn cached_public_is_complete_short_page_is_true() {
        assert!(cached_public_is_complete(5, 20));
    }
}
