use crate::SurrealClient;
use crate::queries::rows_to;
use crate::queries::timeline::NOTE_WITH_AUTHOR_FIELDS;
use crate::queries::timeline::NoteWithAuthor;
use anyhow::Result;

pub async fn get_notes_by_tag(
    client: &SurrealClient,
    tag: &str,
    limit: usize,
) -> Result<Vec<NoteWithAuthor>> {
    let tag_str = tag.to_lowercase();
    let limit = limit.min(100);

    let mut response = client
        .query(format!(
            "
            SELECT {NOTE_WITH_AUTHOR_FIELDS}
            FROM note
            WHERE tags CONTAINS $tag
            ORDER BY id DESC
            LIMIT $limit;
            "
        ))
        .bind(("tag", tag_str))
        .bind(("limit", limit))
        .await?;

    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<NoteWithAuthor>(rows)
}

/// (tag, count) — count は使用回数
pub async fn get_trending_tags(client: &SurrealClient, limit: usize) -> Result<Vec<(String, u64)>> {
    let mut response = client
        .query(
            "
            SELECT value AS tag, COUNT(*) AS count 
            FROM note, array::flatten(tags) AS value
            GROUP BY value
            ORDER BY count DESC
            LIMIT $limit;
            ",
        )
        .bind(("limit", limit))
        .await?;

    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    Ok(rows
        .into_iter()
        .filter_map(|v| {
            let json = v.into_json_value();
            let tag = json.get("tag")?.as_str()?.to_string();
            if tag.is_empty() {
                return None;
            }
            let count = json
                .get("count")
                .and_then(|c| c.as_u64().or_else(|| c.as_i64().map(|i| i.max(0) as u64)))
                .unwrap_or(0);
            Some((tag, count))
        })
        .collect())
}
