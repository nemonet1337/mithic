use axum::{Json, extract::State};
use mithic_core::models::note::NoteId;
use mithic_core::{AppError, Result};
use mithic_db::queries::{NoteWithAuthor, get_global_timeline, get_local_timeline};
use serde::Deserialize;
use shared::Note as NoteDto;

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

fn to_dtos(rows: Vec<NoteWithAuthor>) -> Vec<NoteDto> {
    rows.into_iter()
        .map(|row| note_to_dto(&row.note, actor_to_user(&row.author)))
        .collect()
}

pub async fn local(
    State(state): State<AppState>,
    Json(request): Json<TimelineRequest>,
) -> Result<Json<Vec<NoteDto>>> {
    let limit = request.limit.unwrap_or(30);
    let since_id = parse_id(&request.since_id)?;
    let until_id = parse_id(&request.until_id)?;

    let rows = get_local_timeline(state.surreal(), limit, since_id, until_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(to_dtos(rows)))
}

pub async fn global(
    State(state): State<AppState>,
    Json(request): Json<TimelineRequest>,
) -> Result<Json<Vec<NoteDto>>> {
    let limit = request.limit.unwrap_or(30);
    let since_id = parse_id(&request.since_id)?;
    let until_id = parse_id(&request.until_id)?;

    let rows = get_global_timeline(state.surreal(), limit, since_id, until_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(to_dtos(rows)))
}
