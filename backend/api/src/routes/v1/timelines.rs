//! Timelines: home / local / global / hashtag + trending hashtags

use axum::{
    Extension,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::cache;
use mithic_db::queries::{
    get_global_timeline, get_home_timeline, get_home_timeline_cached, get_local_timeline,
    get_notes_by_tag, get_trending_tags,
};
use serde::Deserialize;
use shared::{Hashtag, Note as NoteDto};

use crate::dto::note_to_dto_full;
use crate::http_cache::{CC_TIMELINE, CC_TRENDING, json_with_cache};
use crate::routes::v1::common::{parse_optional_note_id, rows_to_dtos, PagingQuery};
use crate::state::AppState;

/// home は認証必須 (個人タイムラインのため JSON キャッシュしない / 短命 ID キャッシュは利用)
pub async fn timeline_home(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(paging): Query<PagingQuery>,
) -> Result<axum::Json<Vec<NoteDto>>> {
    let limit = paging.limit.unwrap_or(20).min(100);
    let since_id = parse_optional_note_id(&paging.since_id)?;
    let until_id = parse_optional_note_id(&paging.until_id)?;

    // カーソル無し先頭ページは Dragonfly Sorted Set を優先
    let rows = if since_id.is_none() && until_id.is_none() {
        get_home_timeline_cached(
            state.dragonfly(),
            state.surreal(),
            auth.user_id.to_string(),
            limit,
        )
        .await
    } else {
        get_home_timeline(state.surreal(), &auth.user_id, limit, since_id, until_id).await
    }
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(axum::Json(rows_to_dtos(&state, rows).await))
}

pub async fn timeline_local(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(paging): Query<PagingQuery>,
) -> Result<Response> {
    let limit = paging.limit.unwrap_or(20).min(100);
    let since_id = parse_optional_note_id(&paging.since_id)?;
    let until_id = parse_optional_note_id(&paging.until_id)?;

    let dtos = load_public_timeline(&state, "local", limit, since_id, until_id).await?;
    // 先頭ページのみ HTTP キャッシュ
    if since_id.is_none() && until_id.is_none() {
        Ok(json_with_cache(&headers, dtos, CC_TIMELINE))
    } else {
        Ok(axum::Json(dtos).into_response())
    }
}

pub async fn timeline_global(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(paging): Query<PagingQuery>,
) -> Result<Response> {
    let limit = paging.limit.unwrap_or(20).min(100);
    let since_id = parse_optional_note_id(&paging.since_id)?;
    let until_id = parse_optional_note_id(&paging.until_id)?;

    let dtos = load_public_timeline(&state, "global", limit, since_id, until_id).await?;
    if since_id.is_none() && until_id.is_none() {
        Ok(json_with_cache(&headers, dtos, CC_TIMELINE))
    } else {
        Ok(axum::Json(dtos).into_response())
    }
}

async fn load_public_timeline(
    state: &AppState,
    kind: &str,
    limit: usize,
    since_id: Option<mithic_core::models::note::NoteId>,
    until_id: Option<mithic_core::models::note::NoteId>,
) -> Result<Vec<NoteDto>> {
    // 先頭ページのみ Dragonfly JSON キャッシュ
    if since_id.is_none() && until_id.is_none() {
        let key = cache::timeline_json_key(kind, limit);
        if let Some(dtos) = cache::get_json::<Vec<NoteDto>>(state.dragonfly(), &key).await {
            return Ok(dtos);
        }
    }

    let rows = match kind {
        "local" => get_local_timeline(state.surreal(), limit, since_id, until_id).await,
        "global" => get_global_timeline(state.surreal(), limit, since_id, until_id).await,
        _ => return Err(AppError::Validation(format!("Unknown timeline: {kind}"))),
    }
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let dtos = rows_to_dtos(state, rows).await;
    if since_id.is_none() && until_id.is_none() {
        let key = cache::timeline_json_key(kind, limit);
        let _ = cache::set_json(
            state.dragonfly(),
            &key,
            &dtos,
            cache::TIMELINE_JSON_TTL_SECS,
        )
        .await;
    }
    Ok(dtos)
}

pub async fn timeline_hashtag(
    State(state): State<AppState>,
    Path(tag): Path<String>,
    Query(paging): Query<PagingQuery>,
) -> Result<axum::Json<Vec<NoteDto>>> {
    let limit = paging.limit.unwrap_or(20).min(100);
    let tag = tag.trim_start_matches('#').to_string();
    let notes = get_notes_by_tag(state.surreal(), &tag, limit)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut dtos = Vec::new();
    for note in notes {
        if let Ok(Some(author)) =
            mithic_db::queries::get_actor_by_id(state.surreal(), &note.actor_id).await
        {
            dtos.push(
                note_to_dto_full(&state, &note, crate::dto::actor_to_user(&author)).await,
            );
        }
    }
    Ok(axum::Json(dtos))
}

#[derive(Debug, Deserialize)]
pub struct TrendingQuery {
    pub limit: Option<usize>,
}

pub async fn trending_hashtags(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TrendingQuery>,
) -> Result<Response> {
    let limit = query.limit.unwrap_or(10).min(50);
    let key = cache::trending_json_key(limit);

    if let Some(hashtags) = cache::get_json::<Vec<Hashtag>>(state.dragonfly(), &key).await {
        return Ok(json_with_cache(&headers, hashtags, CC_TRENDING));
    }

    let tags = get_trending_tags(state.surreal(), limit)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let hashtags: Vec<Hashtag> = tags
        .into_iter()
        .map(|(tag, count)| Hashtag {
            tag: if tag.starts_with('#') {
                tag
            } else {
                format!("#{tag}")
            },
            count,
        })
        .collect();

    let _ = cache::set_json(
        state.dragonfly(),
        &key,
        &hashtags,
        cache::TRENDING_JSON_TTL_SECS,
    )
    .await;

    Ok(json_with_cache(&headers, hashtags, CC_TRENDING))
}

use axum::response::IntoResponse;
