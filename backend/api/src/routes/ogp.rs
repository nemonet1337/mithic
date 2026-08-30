use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use mithic_core::models::note::{NoteId, NoteVisibility};
use mithic_core::{AppError, Result};
use mithic_db::queries::{get_actor_by_id, get_actor_by_username, get_note_by_id};

use crate::state::AppState;

fn truncate_chars(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        Some((i, _)) => format!("{}...", &s[..i]),
        None => s.to_string(),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn ogp_html(title: &str, description: &str, url: &str, image: Option<&str>) -> String {
    let avatar_meta = image
        .filter(|u| !u.is_empty())
        .map(|u| format!(r#"<meta property="og:image" content="{}">"#, html_escape(u)))
        .unwrap_or_default();

    format!(
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
        title = html_escape(title),
        url = html_escape(url),
        description = html_escape(description),
        avatar_meta = avatar_meta,
    )
}

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

    // 非公開ノートの OGP は出さない
    if !matches!(
        note.visibility,
        NoteVisibility::Public | NoteVisibility::Home
    ) {
        return Err(AppError::NotFound("Note not found".to_string()));
    }

    let author = get_actor_by_id(state.surreal(), &note.actor_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    let display_name = author.name.as_deref().unwrap_or(&author.username);
    let handle = format!("@{}", author.username);
    let title = format!("{} ({}) のノート", display_name, handle);

    let text = note.text.as_deref().unwrap_or("");
    let description = if text.is_empty() {
        "画像または添付ファイルのみの投稿".to_string()
    } else {
        truncate_chars(text, 150)
    };

    let instance_url = &state.config().instance_url;
    let url = format!("{}/notes/{}", instance_url, note_id);

    Ok(Html(ogp_html(
        &title,
        &description,
        &url,
        author.avatar_url.as_deref(),
    )))
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
    } else {
        truncate_chars(bio, 150)
    };

    let instance_url = &state.config().instance_url;
    let url = format!("{}/profile/{}", instance_url, normalized);

    Ok(Html(ogp_html(
        &title,
        &description,
        &url,
        author.avatar_url.as_deref(),
    )))
}
