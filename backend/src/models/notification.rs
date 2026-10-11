use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{actor::ActorId, note::NoteId};

pub type NotificationId = Ulid;
pub use shared::NotificationType;

/// TEXT カラム → `NotificationType`。DB には snake_case で保存する。
pub fn notif_type_str(nt: NotificationType) -> &'static str {
    match nt {
        NotificationType::Mention => "mention",
        NotificationType::Reply => "reply",
        NotificationType::Renote => "renote",
        NotificationType::Quote => "quote",
        NotificationType::Reaction => "reaction",
        NotificationType::Follow => "follow",
        NotificationType::FollowRequest => "follow_request",
        NotificationType::FollowRequestAccepted => "follow_request_accepted",
        NotificationType::PollEnded => "poll_ended",
        NotificationType::UserSignup => "user_signup",
    }
}

/// TEXT カラム → `NotificationType` (`shared` 型のため `FromStr` は直接 implant できない)
pub fn notif_type_from_str(s: &str) -> anyhow::Result<NotificationType> {
    match s {
        "mention" => Ok(NotificationType::Mention),
        "reply" => Ok(NotificationType::Reply),
        "renote" => Ok(NotificationType::Renote),
        "quote" => Ok(NotificationType::Quote),
        "reaction" => Ok(NotificationType::Reaction),
        "follow" => Ok(NotificationType::Follow),
        "follow_request" => Ok(NotificationType::FollowRequest),
        "follow_request_accepted" => Ok(NotificationType::FollowRequestAccepted),
        "poll_ended" => Ok(NotificationType::PollEnded),
        "user_signup" => Ok(NotificationType::UserSignup),
        other => Err(anyhow::anyhow!("unknown notification type: {other}")),
    }
}

// DB の snake_case フィールドと一致させるため rename しない
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub created_at: DateTime<Utc>,
    pub notification_type: NotificationType,
    pub recipient_id: ActorId,
    pub sender_id: Option<ActorId>,
    pub note_id: Option<NoteId>,
    pub reaction: Option<String>,
    pub is_read: bool,
}

impl Notification {
    pub fn new(
        notification_type: NotificationType,
        recipient_id: ActorId,
        sender_id: Option<ActorId>,
        note_id: Option<NoteId>,
    ) -> Self {
        Self {
            id: NotificationId::generate(),
            created_at: Utc::now(),
            notification_type,
            recipient_id,
            sender_id,
            note_id,
            reaction: None,
            is_read: false,
        }
    }

    pub fn reaction(
        recipient_id: ActorId,
        sender_id: ActorId,
        note_id: NoteId,
        reaction: String,
    ) -> Self {
        let mut notif = Self::new(
            NotificationType::Reaction,
            recipient_id,
            Some(sender_id),
            Some(note_id),
        );
        notif.reaction = Some(reaction);
        notif
    }

    pub fn mention(recipient_id: ActorId, sender_id: ActorId, note_id: NoteId) -> Self {
        Self::new(
            NotificationType::Mention,
            recipient_id,
            Some(sender_id),
            Some(note_id),
        )
    }

    pub fn follow(recipient_id: ActorId, sender_id: ActorId) -> Self {
        Self::new(
            NotificationType::Follow,
            recipient_id,
            Some(sender_id),
            None,
        )
    }

    pub fn mark_as_read(&mut self) {
        self.is_read = true;
    }
}
