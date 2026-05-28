use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::models::note::{Note, NoteId};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{create_note, delete_note, get_actor_by_id, get_note_by_id};
use serde::Deserialize;
use shared::{CreateNoteRequest, Note as NoteDto};

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
