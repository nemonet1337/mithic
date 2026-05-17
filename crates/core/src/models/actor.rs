use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

pub type ActorId = Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActorType {
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    pub id: ActorId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    #[validate(length(min = 1, max = 128))]
    pub username: String,
    pub username_lower: String,
    #[validate(length(min = 1, max = 128))]
    pub name: Option<String>,
    #[validate(length(max = 4096))]
    pub bio: Option<String>,
    pub followers_count: i32,
    pub following_count: i32,
    pub notes_count: i32,
    #[validate(length(max = 512))]
    pub avatar_url: Option<String>,
    #[validate(length(max = 512))]
    pub banner_url: Option<String>,
    pub is_suspended: bool,
    pub is_locked: bool,
    pub is_bot: bool,
    pub is_admin: bool,
    #[validate(length(max = 128))]
    pub host: Option<String>,
    #[validate(length(max = 512))]
    pub inbox: Option<String>,
    #[validate(length(max = 512))]
    pub shared_inbox: Option<String>,
    #[validate(length(max = 512))]
    pub featured: Option<String>,
    #[validate(length(max = 512))]
    pub uri: Option<String>,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub token: Option<String>,
    pub password_hash: Option<String>,
    pub email: Option<String>,
}

impl Actor {
    pub fn new_local(username: String, name: Option<String>) -> Self {
        let now = Utc::now();
        let username_lower = username.to_lowercase();
        Self {
            id: ActorId::new(),
            created_at: now,
            updated_at: None,
            username,
            username_lower,
            name,
            bio: None,
            followers_count: 0,
            following_count: 0,
            notes_count: 0,
            avatar_url: None,
            banner_url: None,
            is_suspended: false,
            is_locked: false,
            is_bot: false,
            is_admin: false,
            host: None,
            inbox: None,
            shared_inbox: None,
            featured: None,
            uri: None,
            public_key: None,
            private_key: None,
            token: None,
            password_hash: None,
            email: None,
        }
    }

    pub fn is_local(&self) -> bool { self.host.is_none() }
    pub fn is_remote(&self) -> bool { self.host.is_some() }

    pub fn actor_uri(&self, instance_url: &str) -> String {
        format!("{}/users/{}", instance_url, self.username)
    }

    pub fn inbox_url(&self, instance_url: &str) -> String {
        format!("{}/users/{}/inbox", instance_url, self.username)
    }

    pub fn outbox_url(&self, instance_url: &str) -> String {
        format!("{}/users/{}/outbox", instance_url, self.username)
    }
}

pub type LocalActor = Actor;
pub type RemoteActor = Actor;

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 128))]
    pub display_name: Option<String>,
    #[validate(length(max = 4096))]
    pub bio: Option<String>,
    pub avatar_id: Option<Ulid>,
    pub header_id: Option<Ulid>,
    pub is_locked: Option<bool>,
    pub is_bot: Option<bool>,
}
