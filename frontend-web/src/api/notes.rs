use serde::Serialize;

use super::client::{ApiError, request};
use crate::models::{CreateNoteRequest, Note};

pub async fn fetch_timeline(
    token: &str,
    kind: &str,
    until_id: Option<&str>,
) -> Result<Vec<Note>, ApiError> {
    let path = match until_id {
        Some(id) => format!("timelines/{}?limit=20&cursor={}", kind, id),
        None => format!("timelines/{}?limit=20", kind),
    };
    request::<Vec<Note>, ()>("GET", &path, Some(token), None).await
}

pub async fn fetch_note(token: &str, id: &str) -> Result<Note, ApiError> {
    request::<Note, ()>("GET", &format!("notes/{}", id), Some(token), None).await
}

pub async fn fetch_replies(token: &str, id: &str) -> Result<Vec<Note>, ApiError> {
    request::<Vec<Note>, ()>(
        "GET",
        &format!("notes/{}/replies", id),
        Some(token),
        None,
    )
    .await
}

pub async fn fetch_quotes(token: &str, id: &str) -> Result<Vec<Note>, ApiError> {
    request::<Vec<Note>, ()>("GET", &format!("notes/{}/quotes", id), Some(token), None).await
}

pub async fn create_note(token: &str, body: &CreateNoteRequest) -> Result<Note, ApiError> {
    request("POST", "notes", Some(token), Some(body)).await
}

pub async fn delete_note(token: &str, id: &str) -> Result<(), ApiError> {
    request::<(), ()>("DELETE", &format!("notes/{}", id), Some(token), None).await
}

pub async fn add_reaction(token: &str, note_id: &str, emoji: &str) -> Result<(), ApiError> {
    #[derive(Serialize)]
    struct Body<'a> {
        reaction: &'a str,
    }
    request::<(), Body>(
        "POST",
        &format!("notes/{}/reactions", note_id),
        Some(token),
        Some(&Body { reaction: emoji }),
    )
    .await
}

pub async fn remove_reaction(token: &str, note_id: &str, emoji: &str) -> Result<(), ApiError> {
    request::<(), ()>(
        "DELETE",
        &format!("notes/{}/reactions/{}", note_id, emoji),
        Some(token),
        None,
    )
    .await
}

pub async fn renote(token: &str, note_id: &str) -> Result<Note, ApiError> {
    request::<Note, ()>(
        "POST",
        &format!("notes/{}/renote", note_id),
        Some(token),
        None,
    )
    .await
}
