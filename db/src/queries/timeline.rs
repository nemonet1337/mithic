use crate::SurrealClient;
use crate::queries::rows_to;
use mithic_core::models::note::{Note, NoteId};

pub async fn get_local_timeline(
    client: &SurrealClient,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<Note>> {
    let limit = limit.min(100);

    let mut query = "
        SELECT 
            *,
            actor_id.id AS actor_id,
            reply_id.id AS reply_id,
            renote_id.id AS renote_id
        FROM note
        WHERE visibility = 'public' 
          AND actor_id.host = None
    "
    .to_string();

    if since_id.is_some() {
        query.push_str(" AND id > $since_id");
    }
    if until_id.is_some() {
        query.push_str(" AND id < $until_id");
    }

    query.push_str(" ORDER BY id DESC LIMIT $limit;");

    let mut q = client.query(&query).bind(("limit", limit));

    if let Some(since) = since_id {
        q = q.bind(("since_id", since.to_string()));
    }
    if let Some(until) = until_id {
        q = q.bind(("until_id", until.to_string()));
    }

    let mut response = q.await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<Note>(rows)
}

pub async fn get_global_timeline(
    client: &SurrealClient,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<Note>> {
    let limit = limit.min(100);

    let mut query = "
        SELECT
            *,
            actor_id.id AS actor_id,
            reply_id.id AS reply_id,
            renote_id.id AS renote_id
        FROM note
        WHERE visibility = 'public'
    "
    .to_string();

    if since_id.is_some() {
        query.push_str(" AND id > $since_id");
    }
    if until_id.is_some() {
        query.push_str(" AND id < $until_id");
    }

    query.push_str(" ORDER BY id DESC LIMIT $limit;");

    let mut q = client.query(&query).bind(("limit", limit));

    if let Some(since) = since_id {
        q = q.bind(("since_id", since.to_string()));
    }
    if let Some(until) = until_id {
        q = q.bind(("until_id", until.to_string()));
    }

    let mut response = q.await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<Note>(rows)
}
