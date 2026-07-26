use mithic_core::models::actor::Actor;
use mithic_core::models::note::{Note, NoteVisibility as CoreVisibility};
use mithic_core::models::notification::NotificationType;
use shared::{
    Note as NoteDto, NoteVisibility, NotificationType as NotifTypeDto, ReactionSummary, User,
};

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
    }
}
