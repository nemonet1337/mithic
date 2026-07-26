//! v1 共通ヘルパー

use mithic_core::models::actor::ActorId;
use mithic_core::models::note::NoteId;
use mithic_core::{AppError, Result};
use mithic_db::queries::NoteWithAuthor;
use serde::Deserialize;
use serde_json::Value;
use shared::Note as NoteDto;

use crate::dto::{actor_to_user, note_to_dto};

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

pub fn rows_to_dtos(rows: Vec<NoteWithAuthor>) -> Vec<NoteDto> {
    rows.into_iter()
        .map(|row| note_to_dto(&row.note, actor_to_user(&row.author)))
        .collect()
}

pub fn normalize_handle(handle: &str) -> String {
    handle.trim().trim_start_matches('@').to_string()
}
