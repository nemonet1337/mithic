use super::client::{ApiError, request, urlencoding_loose};
use serde::Serialize;
use shared::{CreateNoteRequest, Note};

pub async fn fetch_timeline(
    token: &str,
    kind: &str,
    until_id: Option<&str>,
) -> Result<Vec<Note>, ApiError> {
    let path = match until_id {
        Some(id) => format!("timelines/{}?limit=20&until_id={}", kind, id),
        None => format!("timelines/{}?limit=20", kind),
    };
    request::<Vec<Note>, ()>("GET", &path, Some(token), None).await
}

pub async fn create_note(token: &str, body: &CreateNoteRequest) -> Result<Note, ApiError> {
    request("POST", "notes", Some(token), Some(body)).await
}

pub async fn fetch_note(token: &str, id: &str) -> Result<Note, ApiError> {
    request::<Note, ()>("GET", &format!("notes/{}", id), Some(token), None).await
}

pub async fn fetch_replies(token: &str, id: &str) -> Result<Vec<Note>, ApiError> {
    request::<Vec<Note>, ()>("GET", &format!("notes/{}/replies", id), Some(token), None).await
}

pub async fn delete_note(token: &str, id: &str) -> Result<(), ApiError> {
    request::<serde_json::Value, ()>("DELETE", &format!("notes/{}", id), Some(token), None)
        .await
        .map(|_| ())
}

pub async fn add_reaction(
    token: &str,
    note_id: &str,
    emoji: &str,
) -> Result<Vec<shared::ReactionSummary>, ApiError> {
    #[derive(Serialize)]
    struct Body<'a> {
        emoji: &'a str,
    }
    request::<Vec<shared::ReactionSummary>, Body>(
        "POST",
        &format!("notes/{}/reactions", note_id),
        Some(token),
        Some(&Body { emoji }),
    )
    .await
}

pub async fn pin_note(token: &str, note_id: &str) -> Result<(), ApiError> {
    request::<serde_json::Value, ()>("POST", &format!("notes/{note_id}/pin"), Some(token), None)
        .await
        .map(|_| ())
}

pub async fn renote(token: &str, note_id: &str) -> Result<Note, ApiError> {
    request::<Note, ()>(
        "POST",
        &format!("notes/{}/renotes", note_id),
        Some(token),
        None,
    )
    .await
}

pub async fn search_notes(
    token: Option<&str>,
    q: &str,
    limit: usize,
) -> Result<Vec<Note>, ApiError> {
    let path = format!("notes/search?q={}&limit={}", urlencoding_loose(q), limit);
    request::<Vec<Note>, ()>("GET", &path, token, None).await
}

pub async fn fetch_hashtag_timeline(
    token: Option<&str>,
    tag: &str,
    limit: usize,
) -> Result<Vec<Note>, ApiError> {
    let tag = tag.trim_start_matches('#');
    let path = format!(
        "timelines/hashtag/{}?limit={}",
        urlencoding_loose(tag),
        limit
    );
    request::<Vec<Note>, ()>("GET", &path, token, None).await
}

pub async fn fetch_trending(limit: usize) -> Result<Vec<shared::Hashtag>, ApiError> {
    request::<Vec<shared::Hashtag>, ()>(
        "GET",
        &format!("hashtags/trending?limit={limit}"),
        None,
        None,
    )
    .await
}
