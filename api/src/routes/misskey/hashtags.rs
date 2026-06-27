use axum::{Json, extract::State};
use mithic_core::{AppError, Result};
use mithic_db::queries::{get_notes_by_tag, get_trending_tags};

use crate::dto::note_to_dto;
use crate::state::AppState;
use shared::{Hashtag, Note as NoteDto};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashtagTimelineRequest {
    pub tag: String,
    pub limit: Option<usize>,
}

pub async fn hashtag_timeline(
    State(state): State<AppState>,
    Json(request): Json<HashtagTimelineRequest>,
) -> Result<Json<Vec<NoteDto>>> {
    let limit = request.limit.unwrap_or(20).min(100);

    let notes = get_notes_by_tag(state.surreal(), &request.tag, limit)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut dtos = Vec::new();
    for note in notes {
        if let Ok(Some(author)) =
            mithic_db::queries::get_actor_by_id(state.surreal(), &note.actor_id).await
        {
            dtos.push(note_to_dto(&note, crate::dto::actor_to_user(&author)));
        }
    }

    Ok(Json(dtos))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingRequest {
    pub limit: Option<usize>,
}

pub async fn trending(
    State(state): State<AppState>,
    Json(request): Json<TrendingRequest>,
) -> Result<Json<Vec<Hashtag>>> {
    let limit = request.limit.unwrap_or(10).min(50);

    let tags = get_trending_tags(state.surreal(), limit)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let hashtags = tags
        .into_iter()
        .map(|tag| Hashtag {
            tag: format!("#{}", tag),
            count: 0,
        })
        .collect();

    Ok(Json(hashtags))
}
