use serde::{Deserialize, Serialize};

use super::{Note, User};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: String,
    pub created_at: String,
    pub notification_type: NotificationType,
    pub sender: Option<User>,
    pub note: Option<Note>,
    pub reaction: Option<String>,
    pub is_read: bool,
}
