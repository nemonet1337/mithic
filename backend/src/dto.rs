use std::collections::HashMap;

use crate::db::queries::{
    NoteWithAuthor, get_drive_files_by_ids, get_notes_with_authors_by_ids, get_reaction_by_actor,
    get_reactions_by_actor_for_notes,
};
use crate::models::actor::Actor;
use crate::models::file::DriveFile;
use crate::models::note::Note;
use shared::{MediaAttachment, Note as NoteDto, Poll, PollChoice, ReactionSummary, User};

use crate::state::AppState;

pub fn actor_to_user(actor: &Actor) -> User {
    User {
        id: actor.id.to_string(),
        username: actor.username.clone(),
        host: actor.host.clone(),
        display_name: actor.name.clone(),
        bio: actor.bio.clone(),
        avatar_url: actor.avatar_url.clone(),
        banner_url: actor.banner_url.clone(),
        followers_count: actor.followers_count.max(0) as u64,
        following_count: actor.following_count.max(0) as u64,
        notes_count: actor.notes_count.max(0) as u64,
        is_locked: actor.is_locked,
        is_bot: actor.is_bot,
        is_cat: actor.is_cat,
        location: actor.location.clone(),
        birthday: actor.birthday.clone(),
        lang: actor.lang.clone(),
        fields: actor.fields.clone(),
        followed_message: actor.followed_message.clone(),
        reaction_acceptance: actor.reaction_acceptance.clone(),
        created_at: Some(actor.created_at.to_rfc3339()),
    }
}

pub fn apply_my_reactions(summaries: &mut [ReactionSummary], mine: Option<&str>) {
    for r in summaries.iter_mut() {
        r.reacted_by_me = mine.is_some_and(|m| m == r.emoji);
    }
}

pub async fn apply_viewer_reaction(state: &AppState, dto: &mut NoteDto, viewer_id: &str) {
    let mine = get_reaction_by_actor(state.surreal(), &dto.id, viewer_id)
        .await
        .ok()
        .flatten();
    apply_my_reactions(&mut dto.reactions, mine.as_deref());
}

fn attachments_for(
    file_ids: &[String],
    files: &HashMap<String, DriveFile>,
) -> Vec<MediaAttachment> {
    file_ids
        .iter()
        .filter_map(|id| files.get(id).map(drive_file_to_attachment))
        .collect()
}

async fn load_files_map(state: &AppState, file_ids: &[String]) -> HashMap<String, DriveFile> {
    if file_ids.is_empty() {
        return HashMap::new();
    }
    let files = get_drive_files_by_ids(state.surreal(), file_ids)
        .await
        .unwrap_or_default();
    files.into_iter().map(|f| (f.id.to_string(), f)).collect()
}

/// 複数ノートを添付・リノート・閲覧者リアクション込みで DTO 化する (バッチ)
pub async fn notes_to_dtos(
    state: &AppState,
    rows: &[NoteWithAuthor],
    viewer_id: Option<&str>,
) -> Vec<NoteDto> {
    if rows.is_empty() {
        return Vec::new();
    }

    let renote_ids: Vec<String> = rows
        .iter()
        .filter_map(|r| r.note.renote_id.map(|id| id.to_string()))
        .collect();
    let renotes = get_notes_with_authors_by_ids(state.surreal(), &renote_ids)
        .await
        .unwrap_or_default();
    let renote_map: HashMap<String, NoteWithAuthor> = renotes
        .into_iter()
        .map(|row| (row.note.id.to_string(), row))
        .collect();

    let mut file_ids: Vec<String> = rows
        .iter()
        .flat_map(|r| r.note.file_ids.iter().cloned())
        .collect();
    for target in renote_map.values() {
        file_ids.extend(target.note.file_ids.iter().cloned());
    }
    file_ids.sort();
    file_ids.dedup();
    let files = load_files_map(state, &file_ids).await;

    let note_ids: Vec<String> = rows.iter().map(|r| r.note.id.to_string()).collect();
    let mine = match viewer_id {
        Some(vid) => get_reactions_by_actor_for_notes(state.surreal(), vid, &note_ids)
            .await
            .unwrap_or_default(),
        None => HashMap::new(),
    };

    let mut poll_ids: Vec<String> = note_ids.clone();
    poll_ids.extend(renote_map.keys().cloned());
    let polls = load_polls(state, &poll_ids, viewer_id).await;

    rows.iter()
        .map(|row| {
            let mut dto = note_to_dto(&row.note, actor_to_user(&row.author));
            dto.attachments = attachments_for(&row.note.file_ids, &files);
            dto.poll = polls.get(&dto.id).cloned();
            if let Some(rid) = row.note.renote_id {
                if let Some(target) = renote_map.get(&rid.to_string()) {
                    let mut nested = note_to_dto(&target.note, actor_to_user(&target.author));
                    nested.attachments = attachments_for(&target.note.file_ids, &files);
                    nested.poll = polls.get(&nested.id).cloned();
                    dto.renote = Some(Box::new(nested));
                }
            }
            if let Some(emoji) = mine.get(&dto.id) {
                apply_my_reactions(&mut dto.reactions, Some(emoji));
            }
            dto
        })
        .collect()
}

pub fn reaction_summaries_from_map(
    reactions: &std::collections::HashMap<String, i32>,
    mine: Option<&str>,
) -> Vec<ReactionSummary> {
    let mut list: Vec<ReactionSummary> = reactions
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(emoji, count)| ReactionSummary {
            emoji: emoji.clone(),
            count: (*count).max(0) as u64,
            reacted_by_me: mine.is_some_and(|m| m == emoji),
        })
        .collect();
    list.sort_by(|a, b| b.count.cmp(&a.count).then(a.emoji.cmp(&b.emoji)));
    list
}

pub fn drive_file_to_attachment(f: &crate::models::file::DriveFile) -> MediaAttachment {
    MediaAttachment {
        id: f.id.to_string(),
        url: f.url.clone().unwrap_or_default(),
        preview_url: f.thumbnail_url.clone(),
        media_type: f.mime_type.clone(),
        alt: None,
        is_sensitive: false,
    }
}

/// Sync minimal DTO conversion (renote / attachments filled by enrich later)
pub fn note_to_dto(note: &Note, author: User) -> NoteDto {
    let reactions: Vec<ReactionSummary> = note
        .reactions
        .iter()
        .map(|(emoji, count)| ReactionSummary {
            emoji: emoji.clone(),
            count: (*count).max(0) as u64,
            reacted_by_me: false,
        })
        .collect();

    NoteDto {
        id: note.id.to_string(),
        created_at: note.created_at.to_rfc3339(),
        author,
        content: note.text.clone().unwrap_or_default(),
        cw: note.cw.clone(),
        visibility: note.visibility,
        reactions: reactions.into_iter().filter(|r| r.count > 0).collect(),
        reply_count: note.replies_count.max(0) as u64,
        renote_count: note.renote_count.max(0) as u64,
        quote_count: 0,
        attachments: Vec::new(),
        tags: note.tags.clone(),
        is_nsfw: false,
        renote_id: note.renote_id.map(|id| id.to_string()),
        renote: None,
        poll: None,
    }
}

pub async fn note_to_dto_full(state: &AppState, note: &Note, author: User) -> NoteDto {
    let mut files: Vec<String> = note.file_ids.clone();
    let renotes = if let Some(rid) = note.renote_id {
        get_notes_with_authors_by_ids(state.surreal(), &[rid.to_string()])
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    if let Some(target) = renotes.first() {
        files.extend(target.note.file_ids.iter().cloned());
    }
    files.sort();
    files.dedup();
    let file_map = load_files_map(state, &files).await;

    let mut poll_note_ids = vec![note.id.to_string()];
    if let Some(target) = renotes.first() {
        poll_note_ids.push(target.note.id.to_string());
    }
    let polls = load_polls(state, &poll_note_ids, None).await;

    let mut dto = note_to_dto(note, author);
    dto.attachments = attachments_for(&note.file_ids, &file_map);
    dto.poll = polls.get(&dto.id).cloned();
    if let Some(target) = renotes.first() {
        let mut nested = note_to_dto(&target.note, actor_to_user(&target.author));
        nested.attachments = attachments_for(&target.note.file_ids, &file_map);
        nested.poll = polls.get(&nested.id).cloned();
        dto.renote = Some(Box::new(nested));
    }
    dto
}

fn record_tail(v: &serde_json::Value) -> Option<String> {
    let s = v.as_str().or_else(|| {
        v.get("id")
            .or_else(|| v.get("tb"))
            .and_then(|x| x.as_str())
    })?;
    Some(s.rsplit(':').next().unwrap_or(s).to_string())
}

fn votes_u64(v: &serde_json::Value) -> u64 {
    v.as_u64()
        .or_else(|| v.as_i64().map(|n| n.max(0) as u64))
        .unwrap_or(0)
}

async fn load_polls(
    state: &AppState,
    note_ids: &[String],
    viewer_id: Option<&str>,
) -> HashMap<String, Poll> {
    if note_ids.is_empty() {
        return HashMap::new();
    }
    let records: Vec<String> = note_ids.iter().map(|id| format!("note:{id}")).collect();
    let Ok(mut res) = state
        .surreal()
        .query("SELECT * FROM poll WHERE note_id IN $ids;")
        .bind(("ids", records))
        .await
    else {
        return HashMap::new();
    };
    let rows: Vec<surrealdb::types::Value> = res.take(0).unwrap_or_default();
    let mut by_note: HashMap<String, (String, Poll)> = HashMap::new();
    for row in rows {
        let json = row.into_json_value();
        let Some(note_id) = json.get("note_id").and_then(record_tail) else {
            continue;
        };
        let Some(poll_id) = json.get("id").and_then(record_tail) else {
            continue;
        };
        let choices = json
            .get("choices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        let text = c.get("text").and_then(|t| t.as_str())?.to_string();
                        Some(PollChoice {
                            text,
                            votes: c.get("votes").map(votes_u64).unwrap_or(0),
                            voted_by_me: false,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        by_note.insert(
            note_id,
            (
                poll_id,
                Poll {
                    choices,
                    multiple: json.get("multiple").and_then(|v| v.as_bool()).unwrap_or(false),
                },
            ),
        );
    }
    if let Some(vid) = viewer_id {
        if !by_note.is_empty() {
            let poll_records: Vec<String> = by_note
                .values()
                .map(|(id, _)| format!("poll:{id}"))
                .collect();
            if let Ok(mut vote_res) = state
                .surreal()
                .query(
                    "SELECT poll_id, choice_index FROM poll_vote WHERE actor_id = type::record('user', $actor) AND poll_id IN $polls;",
                )
                .bind(("actor", vid.to_string()))
                .bind(("polls", poll_records))
                .await
            {
                let vote_rows: Vec<surrealdb::types::Value> = vote_res.take(0).unwrap_or_default();
                let mut voted: HashMap<String, usize> = HashMap::new();
                for row in vote_rows {
                    let json = row.into_json_value();
                    let Some(pid) = json.get("poll_id").and_then(record_tail) else {
                        continue;
                    };
                    let Some(idx) = json.get("choice_index").and_then(|v| {
                        v.as_u64()
                            .or_else(|| v.as_i64().map(|n| n.max(0) as u64))
                    }) else {
                        continue;
                    };
                    voted.insert(pid, idx as usize);
                }
                for (poll_id, poll) in by_note.values_mut() {
                    if let Some(idx) = voted.get(poll_id) {
                        if let Some(choice) = poll.choices.get_mut(*idx) {
                            choice.voted_by_me = true;
                        }
                    }
                }
            }
        }
    }
    by_note.into_iter().map(|(nid, (_, poll))| (nid, poll)).collect()
}
