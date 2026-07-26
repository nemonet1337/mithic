use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

pub type FollowId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub id: FollowId,
    pub created_at: DateTime<Utc>,
    pub follower_id: ActorId,
    pub followee_id: ActorId,
    pub inbox: Option<String>,
    pub shared_inbox: Option<String>,
    pub is_accepted: bool,
}

pub type FollowRequestId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowRequest {
    pub id: FollowRequestId,
    pub created_at: DateTime<Utc>,
    pub follower_id: ActorId,
    pub followee_id: ActorId,
    pub request_message: Option<String>,
}
