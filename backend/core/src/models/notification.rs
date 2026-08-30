use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{actor::ActorId, note::NoteId};

pub type NotificationId = Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    Mention,
    Reply,
    Renote,
    Quote,
    Reaction,
    Follow,
    FollowRequest,
    FollowRequestAccepted,
    PollEnded,
    UserSignup,
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
