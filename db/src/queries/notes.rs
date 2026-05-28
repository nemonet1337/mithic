use crate::SurrealClient;
use crate::queries::rows_to;
use anyhow::anyhow;
use mithic_core::models::note::{Note, NoteId, NoteVisibility};

fn visibility_str(visibility: NoteVisibility) -> &'static str {
    match visibility {
        NoteVisibility::Public => "public",
        NoteVisibility::Home => "home",
        NoteVisibility::Followers => "followers",
        NoteVisibility::Specified => "specified",
    }
}

pub async fn create_note(client: &SurrealClient, note: &Note) -> anyhow::Result<Note> {
    let id_str = note.id.to_string();
    let actor_id_str = note.actor_id.to_string();
    let reply_id_str = note.reply_id.map(|id| id.to_string());
    let renote_id_str = note.renote_id.map(|id| id.to_string());

    let _response = client
        .query(
            "
            INSERT INTO note {
                id: $id,
                created_at: $created_at,
                text: $text,
                cw: $cw,
                actor_id: type::thing('user', $actor_id),
                visibility: $visibility,
                renote_count: $renote_count,
                replies_count: $replies_count,
                reactions: $reactions,
                reply_id: if $reply_id != None { type::thing('note', $reply_id) } else { None },
                renote_id: if $renote_id != None { type::thing('note', $renote_id) } else { None },
                file_ids: $file_ids,
                tags: $tags,
                has_poll: $has_poll
            };
        ",
        )
        .bind(("id", id_str))
        .bind(("created_at", note.created_at))
        .bind(("text", note.text.clone()))
        .bind(("cw", note.cw.clone()))
        .bind(("actor_id", actor_id_str))
        .bind(("visibility", visibility_str(note.visibility)))
        .bind(("renote_count", note.renote_count))
        .bind(("replies_count", note.replies_count))
        .bind(("reactions", note.reactions.clone()))
        .bind(("reply_id", reply_id_str))
        .bind(("renote_id", renote_id_str))
        .bind(("file_ids", note.file_ids.clone()))
        .bind(("tags", note.tags.clone()))
        .bind(("has_poll", note.has_poll))
        .await?;

    get_note_by_id(client, &note.id)
        .await?
        .ok_or_else(|| anyhow!("Failed to retrieve created note"))
}

pub async fn get_note_by_id(client: &SurrealClient, id: &NoteId) -> anyhow::Result<Option<Note>> {
    let id_str = id.to_string();
    let mut response = client
        .query(
            "
            SELECT 
                *,
                actor_id.id AS actor_id,
                reply_id.id AS reply_id,
                renote_id.id AS renote_id
            FROM note 
            WHERE id = $id 
            LIMIT 1;
        ",
        )
        .bind(("id", id_str))
        .await?;

    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    Ok(rows_to::<Note>(rows)?.into_iter().next())
}

pub async fn delete_note(client: &SurrealClient, id: &NoteId) -> anyhow::Result<()> {
    let id_str = id.to_string();
    client
        .query("DELETE note WHERE id = $id;")
        .bind(("id", id_str))
        .await?;
    Ok(())
}
