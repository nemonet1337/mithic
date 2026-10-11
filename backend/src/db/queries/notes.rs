use crate::db::Db;
use crate::db::queries::actors::parse_actor_id;
use crate::models::note::{Note, NoteId, visibility_from_str, visibility_str};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::collections::HashMap;
use std::str::FromStr;

/// notes テーブルの取得カラム。`FromRow` は名前で対応づけるので並び順は問わない。
pub(crate) const NOTE_COLUMNS: &str = "
    id, created_at, reply_id, renote_id, text, cw, actor_id,
    renote_count, replies_count, reactions, visibility, uri,
    file_ids, visible_user_ids, mentions, emojis, tags, has_poll, host
";

#[derive(Debug, FromRow)]
pub(crate) struct NoteRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub reply_id: Option<String>,
    pub renote_id: Option<String>,
    pub text: Option<String>,
    pub cw: Option<String>,
    pub actor_id: String,
    pub renote_count: i32,
    pub replies_count: i32,
    pub reactions: sqlx::types::Json<HashMap<String, i32>>,
    pub visibility: String,
    pub uri: Option<String>,
    pub file_ids: Vec<String>,
    pub visible_user_ids: Vec<String>,
    pub mentions: Vec<String>,
    pub emojis: Vec<String>,
    pub tags: Vec<String>,
    pub has_poll: bool,
    pub host: Option<String>,
}

fn parse_note_id(raw: &str) -> anyhow::Result<NoteId> {
    ulid::Ulid::from_str(raw).map_err(|e| anyhow::anyhow!("invalid note id '{raw}': {e}"))
}

fn id_vec(raw: Vec<String>) -> anyhow::Result<Vec<ulid::Ulid>> {
    raw.iter().map(|s| parse_note_id(s)).collect()
}

impl TryFrom<NoteRow> for Note {
    type Error = anyhow::Error;

    fn try_from(row: NoteRow) -> anyhow::Result<Self> {
        Ok(Note {
            id: parse_note_id(&row.id)?,
            created_at: row.created_at,
            reply_id: row.reply_id.as_deref().map(parse_note_id).transpose()?,
            renote_id: row.renote_id.as_deref().map(parse_note_id).transpose()?,
            text: row.text,
            cw: row.cw,
            actor_id: parse_actor_id(&row.actor_id)?,
            renote_count: row.renote_count,
            replies_count: row.replies_count,
            reactions: row.reactions.0,
            visibility: visibility_from_str(&row.visibility)?,
            uri: row.uri,
            file_ids: row.file_ids,
            visible_user_ids: id_vec(row.visible_user_ids)?,
            mentions: id_vec(row.mentions)?,
            emojis: row.emojis,
            tags: row.tags,
            has_poll: row.has_poll,
            host: row.host,
        })
    }
}

pub async fn create_note(db: &Db, note: &Note) -> anyhow::Result<Note> {
    let reply_id = note.reply_id.map(|id| id.to_string());
    let renote_id = note.renote_id.map(|id| id.to_string());
    let mentions: Vec<String> = note.mentions.iter().map(|id| id.to_string()).collect();
    let visible_user_ids: Vec<String> = note
        .visible_user_ids
        .iter()
        .map(|id| id.to_string())
        .collect();
    let tags: Vec<String> = note.tags.iter().map(|t| t.to_lowercase()).collect();

    let row = sqlx::query_as::<_, NoteRow>(&format!(
        "INSERT INTO notes (
            id, created_at, text, cw, actor_id, visibility,
            renote_count, replies_count, reactions,
            reply_id, renote_id,
            file_ids, tags, has_poll, uri, host,
            mentions, emojis, visible_user_ids
        ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9,
            $10, $11,
            $12, $13, $14, $15, $16,
            $17, $18, $19
        )
        RETURNING {NOTE_COLUMNS}"
    ))
    .bind(note.id.to_string())
    .bind(note.created_at)
    .bind(&note.text)
    .bind(&note.cw)
    .bind(note.actor_id.to_string())
    .bind(visibility_str(note.visibility))
    .bind(note.renote_count)
    .bind(note.replies_count)
    .bind(sqlx::types::Json(&note.reactions))
    .bind(reply_id)
    .bind(renote_id)
    .bind(&note.file_ids)
    .bind(&tags)
    .bind(note.has_poll)
    .bind(&note.uri)
    .bind(&note.host)
    .bind(&mentions)
    .bind(&note.emojis)
    .bind(&visible_user_ids)
    .fetch_one(db)
    .await?;

    Note::try_from(row)
}

pub async fn get_note_by_id(db: &Db, id: &NoteId) -> anyhow::Result<Option<Note>> {
    let row =
        sqlx::query_as::<_, NoteRow>(&format!("SELECT {NOTE_COLUMNS} FROM notes WHERE id = $1"))
            .bind(id.to_string())
            .fetch_optional(db)
            .await?;

    row.map(Note::try_from).transpose()
}

pub async fn delete_note(db: &Db, id: &NoteId) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM notes WHERE id = $1")
        .bind(id.to_string())
        .execute(db)
        .await?;
    Ok(())
}

pub async fn get_note_by_uri(db: &Db, uri: &str) -> anyhow::Result<Option<Note>> {
    let row = sqlx::query_as::<_, NoteRow>(&format!(
        "SELECT {NOTE_COLUMNS} FROM notes WHERE uri = $1 LIMIT 1"
    ))
    .bind(uri)
    .fetch_optional(db)
    .await?;

    row.map(Note::try_from).transpose()
}
