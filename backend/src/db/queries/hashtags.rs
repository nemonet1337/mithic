use crate::db::Db;
use crate::db::queries::notes::{NOTE_COLUMNS, NoteRow};
use crate::db::queries::timeline::{NoteWithAuthor, attach_authors};
use anyhow::Result;

pub async fn get_notes_by_tag(db: &Db, tag: &str, limit: usize) -> Result<Vec<NoteWithAuthor>> {
    let tag_str = tag.to_lowercase();
    let limit = limit.min(100) as i64;

    let notes = sqlx::query_as::<_, NoteRow>(&format!(
        "SELECT {NOTE_COLUMNS} FROM notes
         WHERE $1 = ANY(tags)
         ORDER BY id DESC
         LIMIT $2"
    ))
    .bind(tag_str)
    .bind(limit)
    .fetch_all(db)
    .await?;

    let notes = notes
        .into_iter()
        .map(crate::models::note::Note::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    attach_authors(db, notes).await
}

/// (tag, count) — count は使用回数
pub async fn get_trending_tags(db: &Db, limit: usize) -> Result<Vec<(String, u64)>> {
    let rows: Vec<Vec<String>> = sqlx::query_scalar(
        "SELECT tags FROM notes
         WHERE array_length(tags, 1) > 0
         ORDER BY id DESC
         LIMIT 1000",
    )
    .fetch_all(db)
    .await?;

    let mut counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for tags in rows {
        for tag in tags.iter().filter(|t| !t.is_empty()) {
            *counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    let mut list: Vec<(String, u64)> = counts.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    list.truncate(limit);
    Ok(list)
}
