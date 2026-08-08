use mithic_core::models::actor::Actor;
use mithic_core::models::file::FileId;
use mithic_core::models::note::{Note, NoteVisibility as CoreVisibility};
use mithic_core::models::notification::NotificationType;
use mithic_db::queries::{get_actor_by_id, get_drive_file, get_note_by_id};
use shared::{
    MediaAttachment, Note as NoteDto, NoteVisibility, NotificationType as NotifTypeDto,
    ReactionSummary, User,
};

use crate::state::AppState;

pub fn notif_type_to_dto(nt: NotificationType) -> NotifTypeDto {
    match nt {
        NotificationType::Mention => NotifTypeDto::Mention,
        NotificationType::Reply => NotifTypeDto::Reply,
        NotificationType::Renote => NotifTypeDto::Renote,
        NotificationType::Quote => NotifTypeDto::Quote,
        NotificationType::Reaction => NotifTypeDto::Reaction,
        NotificationType::Follow => NotifTypeDto::Follow,
        NotificationType::FollowRequest => NotifTypeDto::FollowRequest,
        NotificationType::FollowRequestAccepted => NotifTypeDto::FollowRequestAccepted,
        NotificationType::PollEnded => NotifTypeDto::PollEnded,
        NotificationType::UserSignup => NotifTypeDto::UserSignup,
    }
}

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
    }
}

fn map_visibility(visibility: CoreVisibility) -> NoteVisibility {
    match visibility {
        CoreVisibility::Public => NoteVisibility::Public,
        CoreVisibility::Home => NoteVisibility::Home,
        CoreVisibility::Followers => NoteVisibility::Followers,
        CoreVisibility::Specified => NoteVisibility::Specified,
    }
}

pub fn visibility_from_dto(visibility: NoteVisibility) -> CoreVisibility {
    match visibility {
        NoteVisibility::Public => CoreVisibility::Public,
        NoteVisibility::Home => CoreVisibility::Home,
        NoteVisibility::Followers => CoreVisibility::Followers,
        NoteVisibility::Specified => CoreVisibility::Specified,
    }
}

pub fn drive_file_to_attachment(f: &mithic_core::models::file::DriveFile) -> MediaAttachment {
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
        visibility: map_visibility(note.visibility),
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

async fn load_attachments(state: &AppState, file_ids: &[String]) -> Vec<MediaAttachment> {
    let mut out = Vec::with_capacity(file_ids.len());
    for raw in file_ids {
        let Ok(fid) = raw.parse::<FileId>() else {
            continue;
        };
        if let Ok(Some(file)) = get_drive_file(state.surreal(), &fid).await {
            out.push(drive_file_to_attachment(&file));
        }
    }
    out
}

/// Fill attachments + one-level renote.
/// ponytail: N+1; batch/join later if TL is slow.
pub async fn enrich_note_dto(state: &AppState, note: &Note, mut dto: NoteDto) -> NoteDto {
    if !note.file_ids.is_empty() {
        dto.attachments = load_attachments(state, &note.file_ids).await;
    }

    if let Some(renote_id) = note.renote_id {
        if let Ok(Some(target)) = get_note_by_id(state.surreal(), &renote_id).await {
            if let Ok(Some(target_author)) =
                get_actor_by_id(state.surreal(), &target.actor_id).await
            {
                let mut nested = note_to_dto(&target, actor_to_user(&target_author));
                if !target.file_ids.is_empty() {
                    nested.attachments = load_attachments(state, &target.file_ids).await;
                }
                dto.renote = Some(Box::new(nested));
            }
        }
    }

    dto
}

pub async fn note_to_dto_full(state: &AppState, note: &Note, author: User) -> NoteDto {
    let dto = note_to_dto(note, author);
    enrich_note_dto(state, note, dto).await
}
