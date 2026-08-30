use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type ActorId = Ulid;

// DB の snake_case フィールドと一致させるため rename しない
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: ActorId,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    pub username: String,
    pub username_lower: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub followers_count: i32,
    #[serde(default)]
    pub following_count: i32,
    #[serde(default)]
    pub notes_count: i32,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub banner_url: Option<String>,
    #[serde(default)]
    pub is_suspended: bool,
    #[serde(default)]
    pub is_locked: bool,
    #[serde(default)]
    pub is_bot: bool,
    #[serde(default)]
    pub is_cat: bool,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub birthday: Option<String>,
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub fields: Vec<ProfileField>,
    #[serde(default)]
    pub followed_message: Option<String>,
    #[serde(default)]
    pub reaction_acceptance: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub inbox: Option<String>,
    #[serde(default)]
    pub shared_inbox: Option<String>,
    #[serde(default)]
    pub featured: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub public_key: Option<String>,
    #[serde(default)]
    pub private_key: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub password_hash: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub totp_secret: Option<String>,
    #[serde(default)]
    pub totp_verified: bool,
}

impl Actor {
    pub fn new_local(username: String, name: Option<String>) -> Self {
        let now = Utc::now();
        let username_lower = username.to_lowercase();
        Self {
            id: ActorId::generate(),
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
            is_cat: false,
            is_admin: false,
            location: None,
            birthday: None,
            lang: None,
            fields: Vec::new(),
            followed_message: None,
            reaction_acceptance: None,
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
            totp_secret: None,
            totp_verified: false,
        }
    }

    /// リモート (連合) アクター用。private_key / password / token は持たない。
    pub fn new_remote(username: String, name: Option<String>, host: String, uri: String) -> Self {
        let mut actor = Self::new_local(username, name);
        actor.host = Some(host);
        actor.uri = Some(uri);
        actor.private_key = None;
        actor.password_hash = None;
        actor.token = None;
        actor.email = None;
        actor.totp_secret = None;
        actor.totp_verified = false;
        actor
    }

    pub fn is_local(&self) -> bool {
        self.host.is_none()
    }
    pub fn is_remote(&self) -> bool {
        self.host.is_some()
    }

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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileField {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_id: Option<Ulid>,
    pub header_id: Option<Ulid>,
    pub is_locked: Option<bool>,
    pub is_bot: Option<bool>,
}
