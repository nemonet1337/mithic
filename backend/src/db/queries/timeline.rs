use crate::db::Db;
use crate::db::queries::actors::{ACTOR_COLUMNS, ActorRow};
use crate::db::queries::notes::{NOTE_COLUMNS, NoteRow};
use crate::models::actor::Actor;
use crate::models::actor::ActorId;
use crate::models::note::{Note, NoteId};

/// 著者を同梱したノート行 (N+1 解消用)
#[derive(Debug, Clone)]
pub struct NoteWithAuthor {
    pub note: Note,
    pub author: Actor,
}

/// notes を取得し、著者を `id = ANY($1)` で一括取得して Rust 側で zip する。
async fn fetch_notes(
    db: &Db,
    base_where: &str,
    user_id: Option<&str>,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    let limit = limit.min(100) as i64;

    // base_where が $1 を使うのは user_id を取るクエリだけ。
    let mut next = if user_id.is_some() { 2 } else { 1 };
    let mut sql = format!("SELECT {NOTE_COLUMNS} FROM notes WHERE {base_where}");
    if since_id.is_some() {
        sql.push_str(&format!(" AND id > ${next}"));
        next += 1;
    }
    if until_id.is_some() {
        sql.push_str(&format!(" AND id < ${next}"));
        next += 1;
    }
    sql.push_str(&format!(" ORDER BY id DESC LIMIT ${next}"));

    let mut q = sqlx::query_as::<_, NoteRow>(&sql);
    if let Some(user_id) = user_id {
        q = q.bind(user_id.to_string());
    }
    if let Some(since) = since_id {
        q = q.bind(since.to_string());
    }
    if let Some(until) = until_id {
        q = q.bind(until.to_string());
    }
    let rows = q.bind(limit).fetch_all(db).await?;
    let notes = rows
        .into_iter()
        .map(Note::try_from)
        .collect::<anyhow::Result<Vec<_>>>()?;

    attach_authors(db, notes).await
}

/// `NoteRow` をまとめて `Note` へ変換する
fn rows_to_notes(rows: Vec<NoteRow>) -> anyhow::Result<Vec<Note>> {
    rows.into_iter().map(Note::try_from).collect()
}

/// ノート群の著者を一括取得して `NoteWithAuthor` にまとめる。
pub async fn attach_authors(db: &Db, notes: Vec<Note>) -> anyhow::Result<Vec<NoteWithAuthor>> {
    if notes.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<String> = notes.iter().map(|n| n.actor_id.to_string()).collect();
    let rows = sqlx::query_as::<_, ActorRow>(&format!(
        "SELECT {ACTOR_COLUMNS} FROM users WHERE id = ANY($1)"
    ))
    .bind(&ids)
    .fetch_all(db)
    .await?;

    let mut authors: std::collections::HashMap<String, Actor> =
        std::collections::HashMap::with_capacity(rows.len());
    for row in rows {
        if let Ok(actor) = Actor::try_from(row) {
            authors.insert(actor.id.to_string(), actor);
        }
    }

    Ok(notes
        .into_iter()
        .filter_map(|note| {
            authors
                .get(&note.actor_id.to_string())
                .cloned()
                .map(|author| NoteWithAuthor { note, author })
        })
        .collect())
}

pub async fn get_local_timeline(
    db: &Db,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    fetch_notes(
        db,
        "visibility = 'public' AND host IS NULL",
        None,
        limit,
        since_id,
        until_id,
    )
    .await
}

pub async fn get_global_timeline(
    db: &Db,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    fetch_notes(db, "visibility = 'public'", None, limit, since_id, until_id).await
}

pub async fn get_home_timeline(
    db: &Db,
    user_id: &ActorId,
    limit: usize,
    since_id: Option<NoteId>,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    // 意味論を変えないこと: 自分のノートは visibility 問わず、フォロー相手は
    // specified 以外、specified は visible_user_ids に含まれる場合のみ。
    // サブクエリ内の `follower_id` は follows 側を指すので外側の actor_id は修飾不要。
    fetch_notes(
        db,
        "(actor_id = $1
           OR (
             actor_id IN (SELECT followee_id FROM follows WHERE follower_id = $1 AND is_accepted)
             AND visibility <> 'specified'
           )
           OR (
             visibility = 'specified'
             AND $1 = ANY(visible_user_ids)
           ))",
        Some(&user_id.to_string()),
        limit,
        since_id,
        until_id,
    )
    .await
}

/// 指定ユーザーのノート一覧 (著者同梱)。
/// 公開タイムライン向け: public / home のみ (followers/specified は除外)。
pub async fn get_user_notes(
    db: &Db,
    user_id: &ActorId,
    limit: usize,
    until_id: Option<NoteId>,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    fetch_notes(
        db,
        "actor_id = $1 AND (visibility = 'public' OR visibility = 'home')",
        Some(&user_id.to_string()),
        limit,
        None,
        until_id,
    )
    .await
}

/// 指定ノートへの返信一覧 (著者同梱)
pub async fn get_note_replies(
    db: &Db,
    note_id: &NoteId,
    limit: usize,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    let limit = limit.min(100) as i64;
    let notes = sqlx::query_as::<_, NoteRow>(&format!(
        "SELECT {NOTE_COLUMNS} FROM notes
         WHERE reply_id = $1
         ORDER BY id ASC LIMIT $2"
    ))
    .bind(note_id.to_string())
    .bind(limit)
    .fetch_all(db)
    .await?;
    let notes = rows_to_notes(notes)?;

    attach_authors(db, notes).await
}

/// 指定ノートの引用 (テキスト付きリノート) 一覧 (著者同梱)
pub async fn get_note_quotes(
    db: &Db,
    note_id: &NoteId,
    limit: usize,
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    let limit = limit.min(100) as i64;
    let notes = sqlx::query_as::<_, NoteRow>(&format!(
        "SELECT {NOTE_COLUMNS} FROM notes
         WHERE renote_id = $1 AND text IS NOT NULL
         ORDER BY id DESC LIMIT $2"
    ))
    .bind(note_id.to_string())
    .bind(limit)
    .fetch_all(db)
    .await?;
    let notes = rows_to_notes(notes)?;

    attach_authors(db, notes).await
}

pub async fn get_notes_with_authors_by_ids(
    db: &Db,
    ids: &[String],
) -> anyhow::Result<Vec<NoteWithAuthor>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let notes = sqlx::query_as::<_, NoteRow>(&format!(
        "SELECT {NOTE_COLUMNS} FROM notes WHERE id = ANY($1)"
    ))
    .bind(ids)
    .fetch_all(db)
    .await?;
    let notes = rows_to_notes(notes)?;

    let mut rows = attach_authors(db, notes).await?;
    rows.sort_by_key(|row| std::cmp::Reverse(row.note.id));
    Ok(rows)
}
