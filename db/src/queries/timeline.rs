use serde::Deserialize;

use crate::SurrealClient;
use crate::queries::rows_to;
use mithic_core::models::actor::Actor;
use mithic_core::models::note::{Note, NoteId};

/// 著者を同梱したノート行 (N+1 解消用)
#[derive(Debug, Clone, Deserialize)]
pub struct NoteWithAuthor {
    #[serde(flatten)]
    pub note: Note,
    pub author: Actor,
}

/// SELECT 句: ノート本体 + リンク先 ID の平坦化 + 著者レコードの同梱
const NOTE_WITH_AUTHOR_FIELDS: &str = "
    *,
    actor_id.id AS actor_id,
    reply_id.id AS reply_id,
    renote_id.id AS renote_id,
    actor_id.* AS author
";

async fn fetch_notes(
    client: &SurrealClient,
    base_where: &str,
    user_id: Option<String>,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    let limit = limit.min(100);

    let mut query = format!("SELECT {NOTE_WITH_AUTHOR_FIELDS} FROM note WHERE {base_where}");

    if since_id.is_some() {
        query.push_str(" AND id > $since_id");
    }
    if until_id.is_some() {
        query.push_str(" AND id < $until_id");
    }
    query.push_str(" ORDER BY id DESC LIMIT $limit;");

    let mut q = client.query(&query).bind(("limit", limit));

    if let Some(user_id) = user_id {
        q = q.bind(("user_id", user_id));
    }
    if let Some(since) = since_id {
        q = q.bind(("since_id", since.to_string()));
    }
    if let Some(until) = until_id {
        q = q.bind(("until_id", until.to_string()));
    }

    let mut response = q.await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<NoteWithAuthor>(rows)
}

pub async fn get_local_timeline(
    client: &SurrealClient,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    fetch_notes(
        client,
        "visibility = 'public' AND actor_id.host = None",
        None,
        limit,
        since_id,
        until_id,
    )
    .await
}

pub async fn get_global_timeline(
    client: &SurrealClient,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    fetch_notes(
        client,
        "visibility = 'public'",
        None,
        limit,
        since_id,
        until_id,
    )
    .await
}

pub async fn get_home_timeline(
    client: &SurrealClient,
    user_id: &mithic_core::models::actor::ActorId,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    fetch_notes(
        client,
        "(actor_id = type::record('user', $user_id)
           OR actor_id IN (SELECT VALUE out FROM follow WHERE in = type::record('user', $user_id)))",
        Some(user_id.to_string()),
        limit,
        since_id,
        until_id,
    )
    .await
}

/// 指定ユーザーのノート一覧 (著者同梱)
pub async fn get_user_notes(
    client: &SurrealClient,
    user_id: &mithic_core::models::actor::ActorId,
    limit: usize,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    fetch_notes(
        client,
        "actor_id = type::record('user', $user_id)",
        Some(user_id.to_string()),
        limit,
        None,
        until_id,
    )
    .await
}

/// 指定ノートへの返信一覧 (著者同梱)
pub async fn get_note_replies(
    client: &SurrealClient,
    note_id: &NoteId,
    limit: usize,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    let limit = limit.min(100);
    let mut response = client
        .query(format!(
            "SELECT {NOTE_WITH_AUTHOR_FIELDS} FROM note
             WHERE reply_id = type::record('note', $note_id)
             ORDER BY id ASC LIMIT $limit;"
        ))
        .bind(("note_id", note_id.to_string()))
        .bind(("limit", limit))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<NoteWithAuthor>(rows)
}

/// 指定ノートの引用 (テキスト付きリノート) 一覧 (著者同梱)
pub async fn get_note_quotes(
    client: &SurrealClient,
    note_id: &NoteId,
    limit: usize,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    let limit = limit.min(100);
    let mut response = client
        .query(format!(
            "SELECT {NOTE_WITH_AUTHOR_FIELDS} FROM note
             WHERE renote_id = type::record('note', $note_id) AND text != None
             ORDER BY id DESC LIMIT $limit;"
        ))
        .bind(("note_id", note_id.to_string()))
        .bind(("limit", limit))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<NoteWithAuthor>(rows)
}
