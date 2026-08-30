use std::collections::HashMap;

use crate::db::queries::{
    NoteWithAuthor, get_drive_files_by_ids, get_notes_with_authors_by_ids, get_reaction_by_actor,
    get_reactions_by_actor_for_notes,
};
use crate::models::actor::Actor;
use crate::models::file::DriveFile;
use crate::models::note::Note;
use shared::{MediaAttachment, Note as NoteDto, ReactionSummary, User};

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

    rows.iter()
        .map(|row| {
            let mut dto = note_to_dto(&row.note, actor_to_user(&row.author));
            dto.attachments = attachments_for(&row.note.file_ids, &files);
            if let Some(rid) = row.note.renote_id {
                if let Some(target) = renote_map.get(&rid.to_string()) {
                    let mut nested = note_to_dto(&target.note, actor_to_user(&target.author));
                    nested.attachments = attachments_for(&target.note.file_ids, &files);
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
    let reactions = note
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
        reactions,
        reply_count: note.replies_count.max(0) as u64,
        renote_count: note.renote_count.max(0) as u64,
        quote_count: 0,
        attachments: Vec::new(),
        tags: note.tags.clone(),
        is_nsfw: false,
        renote_id: note.renote_id.map(|id| id.to_string()),
        renote: None,
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

    let mut dto = note_to_dto(note, author);
    dto.attachments = attachments_for(&note.file_ids, &file_map);
    if let Some(target) = renotes.first() {
        let mut nested = note_to_dto(&target.note, actor_to_user(&target.author));
        nested.attachments = attachments_for(&target.note.file_ids, &file_map);
        dto.renote = Some(Box::new(nested));
    }
    dto
}
