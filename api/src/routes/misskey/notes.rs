use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::models::note::{Note, NoteId, NoteVisibility};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::cache;
use mithic_db::queries::{
    add_favorite, add_reaction, create_note, delete_note, get_actor_by_id, get_home_timeline,
    get_note_by_id, remove_favorite, remove_reaction,
};
use serde::Deserialize;
use shared::{CreateNoteRequest, Note as NoteDto, ReactionRequest};

use crate::dto::{actor_to_user, note_to_dto};
use crate::state::AppState;

fn parse_note_id(raw: &str) -> Result<NoteId> {
    raw.parse::<NoteId>()
        .map_err(|_| AppError::Validation("Invalid note id".to_string()))
}

/// Note DTO キャッシュ (TTL: 1h)
const NOTE_CACHE_TTL: u64 = 3600;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteIdRequest {
    pub note_id: String,
}

pub async fn create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<CreateNoteRequest>,
) -> Result<Json<NoteDto>> {
    let dto = crate::services::note::create_note_service(&state, auth.user_id, request).await?;
    let note_key = format!("note:{}", dto.id);
    let _ = cache::set_json(state.dragonfly(), &note_key, &dto, NOTE_CACHE_TTL).await;
    Ok(Json(dto))
}

pub async fn show(
    State(state): State<AppState>,
    Json(request): Json<NoteIdRequest>,
) -> Result<Json<NoteDto>> {
    let note_id = parse_note_id(&request.note_id)?;
    let note_key = format!("note:{}", note_id);

    if let Some(dto) = cache::get_json::<NoteDto>(state.dragonfly(), &note_key).await {
        return Ok(Json(dto));
    }

    let note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    let author = get_actor_by_id(state.surreal(), &note.actor_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    let dto = note_to_dto(&note, actor_to_user(&author));
    let _ = cache::set_json(state.dragonfly(), &note_key, &dto, NOTE_CACHE_TTL).await;

    Ok(Json(dto))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<NoteIdRequest>,
) -> Result<StatusCode> {
    let note_id = parse_note_id(&request.note_id)?;

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

    let note_key = format!("note:{}", note_id);
    let _ = cache::delete(state.dragonfly(), &note_key).await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_reaction(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<ReactionRequest>,
) -> Result<StatusCode> {
    let note_id = parse_note_id(&request.note_id)?;

    let _note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    add_reaction(
        state.surreal(),
        &note_id.to_string(),
        &auth.user_id.to_string(),
        &request.reaction,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_reaction(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<ReactionRequest>,
) -> Result<StatusCode> {
    let note_id = parse_note_id(&request.note_id)?;

    let _note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    remove_reaction(
        state.surreal(),
        &note_id.to_string(),
        &auth.user_id.to_string(),
        &request.reaction,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteRequest {
    pub note_id: String,
}

pub async fn create_favorite(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<FavoriteRequest>,
) -> Result<StatusCode> {
    let note_id = parse_note_id(&request.note_id)?;
    let user_id = auth.user_id;

    let _note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    add_favorite(state.surreal(), &user_id, &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_favorite(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<FavoriteRequest>,
) -> Result<StatusCode> {
    let note_id = parse_note_id(&request.note_id)?;
    let user_id = auth.user_id;

    remove_favorite(state.surreal(), &user_id, &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenoteRequest {
    pub note_id: String,
}

pub async fn renote(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<RenoteRequest>,
) -> Result<Json<NoteDto>> {
    let target_note_id = parse_note_id(&request.note_id)?;
    let my_id = auth.user_id;

    let _target_note = get_note_by_id(state.surreal(), &target_note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    let mut renote = Note::new(my_id, None, NoteVisibility::Public);
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

    let author = get_actor_by_id(state.surreal(), &my_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    Ok(Json(note_to_dto(&created, actor_to_user(&author))))
}

pub async fn unrenote(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<RenoteRequest>,
) -> Result<StatusCode> {
    let target_note_id = parse_note_id(&request.note_id)?;
    let my_id = auth.user_id;

    state
        .surreal()
        .query(
            "
        DELETE note WHERE actor_id = type::record('user', $my_id) AND renote_id = type::record('note', $target_id);
        ",
        )
        .bind(("my_id", my_id.to_string()))
        .bind(("target_id", target_note_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    state
        .surreal()
        .query(
            "UPDATE note SET renote_count = <int>(renote_count OR 1) - 1 WHERE id = type::record('note', $id);",
        )
        .bind(("id", target_note_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinRequest {
    pub note_id: String,
}

pub async fn pin_note(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<PinRequest>,
) -> Result<StatusCode> {
    let note_id = parse_note_id(&request.note_id)?;

    let _note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    state
        .surreal()
        .query(
            "
        SELECT * FROM user_note_pining WHERE user_id = type::record('user', $user) LIMIT 5;
        ",
        )
        .bind(("user", auth.user_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = state
        .surreal()
        .query(
            "
        RELATE (type::record('user', $user)) -> user_note_pining -> (type::record('note', $note));
        ",
        )
        .bind(("user", auth.user_id.to_string()))
        .bind(("note", note_id.to_string()))
        .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unpin_note(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<PinRequest>,
) -> Result<StatusCode> {
    let note_id = parse_note_id(&request.note_id)?;

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

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollVoteRequest {
    pub note_id: String,
    pub choice: usize,
}

pub async fn vote_poll(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<PollVoteRequest>,
) -> Result<StatusCode> {
    let note_id = parse_note_id(&request.note_id)?;

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
        request.choice,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineRequest {
    pub limit: Option<usize>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

pub async fn home_timeline(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<TimelineRequest>,
) -> Result<Json<Vec<NoteDto>>> {
    let user_id = auth.user_id;

    let limit = request.limit.unwrap_or(20).min(100);

    let since_id = match request.since_id {
        Some(ref id) => Some(parse_note_id(id)?),
        None => None,
    };
    let until_id = match request.until_id {
        Some(ref id) => Some(parse_note_id(id)?),
        None => None,
    };

    let rows = get_home_timeline(state.surreal(), &user_id, limit, since_id, until_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let dtos = rows
        .into_iter()
        .map(|row| note_to_dto(&row.note, actor_to_user(&row.author)))
        .collect();

    Ok(Json(dtos))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchNotesRequest {
    pub query: String,
    pub limit: Option<usize>,
}

pub async fn search_notes(
    State(state): State<AppState>,
    Json(request): Json<SearchNotesRequest>,
) -> Result<Json<Vec<NoteDto>>> {
    let limit = request.limit.unwrap_or(20).min(100);

    let mut response = state
        .surreal()
        .query(
            "
            SELECT 
                *,
                actor_id.id AS actor_id,
                reply_id.id AS reply_id,
                renote_id.id AS renote_id
            FROM note
            WHERE text CONTAINS $query
            ORDER BY id DESC
            LIMIT $limit;
            ",
        )
        .bind(("query", request.query.clone()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<surrealdb::types::Value> = response
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let notes: Vec<mithic_core::models::note::Note> = rows
        .into_iter()
        .map(|v| {
            let mut json = v.into_json_value();
            mithic_db::queries::strip_record_prefixes(&mut json);
            serde_json::from_value::<mithic_core::models::note::Note>(json)
                .map_err(|e| AppError::Internal(e.to_string()))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut dtos = Vec::new();
    for note in notes {
        let author = get_actor_by_id(state.surreal(), &note.actor_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;
        dtos.push(note_to_dto(&note, actor_to_user(&author)));
    }

    Ok(Json(dtos))
}
