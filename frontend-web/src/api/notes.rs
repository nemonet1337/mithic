use serde::Serialize;

use super::client::{ApiError, request};
use crate::models::{CreateNoteRequest, Note};

pub async fn fetch_timeline(
    token: &str,
    kind: &str,
    until_id: Option<&str>,
) -> Result<Vec<Note>, ApiError> {
    let path = match until_id {
        Some(id) => format!("v1/timelines/{}?limit=20&until_id={}", kind, id),
        None => format!("v1/timelines/{}?limit=20", kind),
    };
    request::<Vec<Note>, ()>("GET", &path, Some(token), None).await
}

pub async fn fetch_note(token: &str, id: &str) -> Result<Note, ApiError> {
    request::<Note, ()>("GET", &format!("v1/notes/{}", id), Some(token), None).await
}

pub async fn fetch_replies(token: &str, id: &str) -> Result<Vec<Note>, ApiError> {
    request::<Vec<Note>, ()>(
        "GET",
        &format!("v1/notes/{}/replies", id),
        Some(token),
        None,
    )
    .await
}

pub async fn fetch_quotes(token: &str, id: &str) -> Result<Vec<Note>, ApiError> {
    request::<Vec<Note>, ()>("GET", &format!("v1/notes/{}/quotes", id), Some(token), None).await
}

pub async fn create_note(token: &str, body: &CreateNoteRequest) -> Result<Note, ApiError> {
    request("POST", "v1/notes", Some(token), Some(body)).await
}

pub async fn delete_note(token: &str, id: &str) -> Result<(), ApiError> {
    request::<(), ()>("DELETE", &format!("v1/notes/{}", id), Some(token), None).await
}

pub async fn add_reaction(token: &str, note_id: &str, emoji: &str) -> Result<(), ApiError> {
    #[derive(Serialize)]
    struct Body<'a> {
        emoji: &'a str,
    }
    request::<(), Body>(
        "POST",
        &format!("v1/notes/{}/reactions", note_id),
        Some(token),
        Some(&Body { emoji }),
    )
    .await
}

pub async fn remove_reaction(token: &str, note_id: &str, emoji: &str) -> Result<(), ApiError> {
    request::<(), ()>(
        "DELETE",
        &format!("v1/notes/{}/reactions/{}", note_id, emoji),
        Some(token),
        None,
    )
    .await
}

pub async fn renote(token: &str, note_id: &str) -> Result<Note, ApiError> {
    request::<Note, ()>(
        "POST",
        &format!("v1/notes/{}/renotes", note_id),
        Some(token),
        None,
    )
    .await
}
