use crate::db::SurrealClient;
use crate::db::queries::rows_to;
use crate::db::queries::timeline::NOTE_WITH_AUTHOR_FIELDS;
use crate::db::queries::timeline::NoteWithAuthor;
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
            SELECT tags FROM note
            WHERE array::len(tags) > 0
            LIMIT 1000;
            ",
        )
        .await?;

    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    let mut counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for row in rows {
        let Some(tags) = row
            .into_json_value()
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|a| a.to_vec())
        else {
            continue;
        };
        for tag in tags {
            if let Some(tag) = tag.as_str().filter(|t| !t.is_empty()) {
                *counts.entry(tag.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut list: Vec<(String, u64)> = counts.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    list.truncate(limit);
    Ok(list)
}
