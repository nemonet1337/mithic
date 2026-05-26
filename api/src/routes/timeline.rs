use std::collections::HashMap;

use axum::{Json, extract::State};
use mithic_core::models::actor::ActorId;
use mithic_core::models::note::{Note, NoteId};
use mithic_core::{AppError, Result};
use mithic_db::queries::{get_actor_by_id, get_global_timeline, get_local_timeline};
use serde::Deserialize;
use shared::{Note as NoteDto, User};

use crate::dto::{actor_to_user, note_to_dto};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineRequest {
    pub limit: Option<usize>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

fn parse_id(raw: &Option<String>) -> Result<Option<NoteId>> {
    match raw {
        Some(value) => value
            .parse::<NoteId>()
            .map(Some)
            .map_err(|_| AppError::Validation("Invalid note id".to_string())),
        None => Ok(None),
    }
}

async fn resolve_authors(state: &AppState, notes: &[Note]) -> Result<HashMap<ActorId, User>> {
    let mut authors: HashMap<ActorId, User> = HashMap::new();
    for note in notes {
        if authors.contains_key(&note.actor_id) {
            continue;
        }
        if let Some(actor) = get_actor_by_id(state.surreal(), &note.actor_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
        {
            authors.insert(note.actor_id, actor_to_user(&actor));
        }
    }
    Ok(authors)
}

fn to_dtos(notes: Vec<Note>, authors: &HashMap<ActorId, User>) -> Vec<NoteDto> {
    notes
        .into_iter()
        .filter_map(|note| {
            authors
                .get(&note.actor_id)
                .map(|author| note_to_dto(&note, author.clone()))
        })
        .collect()
}

pub async fn local(
    State(state): State<AppState>,
    Json(request): Json<TimelineRequest>,
) -> Result<Json<Vec<NoteDto>>> {
    let limit = request.limit.unwrap_or(30);
    let since_id = parse_id(&request.since_id)?;
    let until_id = parse_id(&request.until_id)?;

    let notes = get_local_timeline(state.surreal(), limit, since_id, until_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let authors = resolve_authors(&state, &notes).await?;
    Ok(Json(to_dtos(notes, &authors)))
}

pub async fn global(
    State(state): State<AppState>,
    Json(request): Json<TimelineRequest>,
) -> Result<Json<Vec<NoteDto>>> {
    let limit = request.limit.unwrap_or(30);
    let since_id = parse_id(&request.since_id)?;
    let until_id = parse_id(&request.until_id)?;

    let notes = get_global_timeline(state.surreal(), limit, since_id, until_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let authors = resolve_authors(&state, &notes).await?;
    Ok(Json(to_dtos(notes, &authors)))
}
