use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::models::note::{Note, NoteId};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{
    add_favorite, add_reaction, create_note, delete_note, get_actor_by_id, get_home_timeline,
    get_note_by_id, remove_favorite, remove_reaction,
};
use serde::Deserialize;
use shared::{CreateNoteRequest, Note as NoteDto, ReactionRequest};

use crate::dto::{actor_to_user, note_to_dto, visibility_from_dto};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteIdRequest {
    pub note_id: String,
}

fn parse_note_id(raw: &str) -> Result<NoteId> {
    raw.parse::<NoteId>()
        .map_err(|_| AppError::Validation("Invalid note id".to_string()))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<CreateNoteRequest>,
) -> Result<Json<NoteDto>> {
    let text = if request.text.is_empty() {
        None
    } else {
        Some(request.text)
    };

    let mut note = Note::new(auth.user_id, text, visibility_from_dto(request.visibility));
    note.cw = request.cw;
    note.file_ids = request.file_ids;
    if let Some(reply_id) = request.reply_id {
        note.reply_id = Some(parse_note_id(&reply_id)?);
    }

    let created = create_note(state.surreal(), &note)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let author = get_actor_by_id(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    Ok(Json(note_to_dto(&created, actor_to_user(&author))))
}

pub async fn show(
    State(state): State<AppState>,
    Json(request): Json<NoteIdRequest>,
) -> Result<Json<NoteDto>> {
    let note_id = parse_note_id(&request.note_id)?;

    let note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    let author = get_actor_by_id(state.surreal(), &note.actor_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    Ok(Json(note_to_dto(&note, actor_to_user(&author))))
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

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_reaction(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<ReactionRequest>,
) -> Result<StatusCode> {
    let note_id = parse_note_id(&request.note_id)?;

    // Check if the note exists
    let _note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    // Add reaction
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

    // Check if the note exists
    let _note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    // Remove reaction
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

    // Check if note exists
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

    let mut renote = Note::new(
        my_id,
        None,
        mithic_core::models::note::NoteVisibility::Public,
    );
    renote.renote_id = Some(target_note_id);

    let created = create_note(state.surreal(), &renote)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    state
        .surreal()
        .query("UPDATE note SET renote_count += 1 WHERE id = $id;")
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

    let mut response = state.surreal()
        .query(
            "
            DELETE note WHERE actor_id = type::thing('user', $my_id) AND renote_id = type::thing('note', $target_id);
            ",
        )
        .bind(("my_id", my_id.to_string()))
        .bind(("target_id", target_note_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<serde_json::Value> = response
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let deleted_notes: Vec<Note> = rows
        .into_iter()
        .map(|v| serde_json::from_value::<Note>(v).map_err(|e| AppError::Internal(e.to_string())))
        .collect::<Result<Vec<Note>>>()?;

    if !deleted_notes.is_empty() {
        state
            .surreal()
            .query("UPDATE note SET renote_count = <int>(renote_count OR 1) - 1 WHERE id = $id;")
            .bind(("id", target_note_id.to_string()))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

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

    let notes = get_home_timeline(state.surreal(), &user_id, limit, since_id, until_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

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
