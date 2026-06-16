use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use mithic_core::models::note::NoteId;
use mithic_core::{AppError, Result};
use mithic_db::queries::{get_actor_by_id, get_actor_by_username, get_note_by_id};

use crate::state::AppState;

pub async fn note_ogp(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let note_id = id
        .parse::<NoteId>()
        .map_err(|_| AppError::Validation("Invalid note id".to_string()))?;

    let note = get_note_by_id(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    let author = get_actor_by_id(state.surreal(), &note.actor_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    let display_name = author.name.as_deref().unwrap_or(&author.username);
    let handle = format!("@{}", author.username);
    let title = format!("{} ({}) のノート", display_name, handle);

    let text = note.text.as_deref().unwrap_or("");
    let description = if text.len() > 150 {
        format!("{}...", &text[..150])
    } else if text.is_empty() {
        "画像または添付ファイルのみの投稿".to_string()
    } else {
        text.to_string()
    };

    let instance_url = &state.config().instance_url;
    let url = format!("{}/notes/{}", instance_url, note_id);
    let avatar_url = author.avatar_url.as_deref().unwrap_or("");

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{title}</title>
    <meta property="og:title" content="{title}">
    <meta property="og:type" content="article">
    <meta property="og:url" content="{url}">
    <meta property="og:description" content="{description}">
    <meta property="og:site_name" content="Mithic">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{description}">
    {avatar_meta}
</head>
<body>
    <article>
        <h1>{title}</h1>
        <p>{description}</p>
    </article>
</body>
</html>"#,
        title = html_escape(&title),
        url = html_escape(&url),
        description = html_escape(&description),
        avatar_meta = if !avatar_url.is_empty() {
            format!(
                r#"<meta property="og:image" content="{}">"#,
                html_escape(avatar_url)
            )
        } else {
            "".to_string()
        }
    );

    Ok(Html(html))
}

pub async fn profile_ogp(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse> {
    let normalized = username.trim().trim_start_matches('@').to_string();
    let author = get_actor_by_username(state.surreal(), &normalized)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let display_name = author.name.as_deref().unwrap_or(&author.username);
    let handle = format!("@{}", author.username);
    let title = format!("{} ({})", display_name, handle);

    let bio = author.bio.as_deref().unwrap_or("");
    let description = if bio.is_empty() {
        format!(
            "Mithic で {} さんのプロフィールをチェックしましょう。",
            handle
        )
    } else if bio.len() > 150 {
        format!("{}...", &bio[..150])
    } else {
        bio.to_string()
    };

    let instance_url = &state.config().instance_url;
    let url = format!("{}/profile/{}", instance_url, normalized);
    let avatar_url = author.avatar_url.as_deref().unwrap_or("");

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{title}</title>
    <meta property="og:title" content="{title}">
    <meta property="og:type" content="profile">
    <meta property="og:url" content="{url}">
    <meta property="og:description" content="{description}">
    <meta property="og:site_name" content="Mithic">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{description}">
    {avatar_meta}
</head>
<body>
    <header>
        <h1>{title}</h1>
        <p>{description}</p>
    </header>
</body>
</html>"#,
        title = html_escape(&title),
        url = html_escape(&url),
        description = html_escape(&description),
        avatar_meta = if !avatar_url.is_empty() {
            format!(
                r#"<meta property="og:image" content="{}">"#,
                html_escape(avatar_url)
            )
        } else {
            "".to_string()
        }
    );

    Ok(Html(html))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
