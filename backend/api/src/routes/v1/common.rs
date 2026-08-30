//! v1 共通ヘルパー

use mithic_core::models::actor::ActorId;
use mithic_core::models::note::NoteId;
use mithic_core::{AppError, Result};
use mithic_db::queries::NoteWithAuthor;
use serde::Deserialize;
use serde_json::Value;
use shared::Note as NoteDto;

use crate::dto::{actor_to_user, apply_viewer_reaction, note_to_dto_full};
use crate::state::AppState;

/// フロントの `request::<(), _>` は JSON `null` を期待する
pub fn ok_null() -> axum::Json<Value> {
    axum::Json(Value::Null)
}

#[derive(Debug, Deserialize)]
pub struct PagingQuery {
    pub limit: Option<usize>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

pub fn parse_optional_note_id(raw: &Option<String>) -> Result<Option<NoteId>> {
    match raw {
        Some(value) => value
            .parse::<NoteId>()
            .map(Some)
            .map_err(|_| AppError::Validation("Invalid note id".to_string())),
        None => Ok(None),
    }
}

pub fn parse_note_id(raw: &str) -> Result<NoteId> {
    raw.parse::<NoteId>()
        .map_err(|_| AppError::Validation("Invalid note id".to_string()))
}

pub fn parse_actor_id(raw: &str) -> Result<ActorId> {
    raw.parse::<ActorId>()
        .map_err(|_| AppError::Validation("Invalid user id".to_string()))
}

pub async fn rows_to_dtos(state: &AppState, rows: Vec<NoteWithAuthor>) -> Vec<NoteDto> {
    rows_to_dtos_for(state, rows, None).await
}

pub async fn rows_to_dtos_for(
    state: &AppState,
    rows: Vec<NoteWithAuthor>,
    viewer_id: Option<&str>,
) -> Vec<NoteDto> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut dto = note_to_dto_full(state, &row.note, actor_to_user(&row.author)).await;
        if let Some(vid) = viewer_id {
            apply_viewer_reaction(state, &mut dto, vid).await;
        }
        out.push(dto);
    }
    out
}

pub fn normalize_handle(handle: &str) -> String {
    handle.trim().trim_start_matches('@').to_string()
}
