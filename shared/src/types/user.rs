use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProfileField {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub username: String,
    pub host: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub followers_count: u64,
    pub following_count: u64,
    pub notes_count: u64,
    pub is_locked: bool,
    pub is_bot: bool,
    #[serde(default)]
    pub is_cat: bool,
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
    pub created_at: Option<String>,
}

impl User {
    pub fn local(username: impl Into<String>, display_name: impl Into<String>) -> Self {
        let username = username.into();
        Self {
            id: username.clone(),
            username,
            host: None,
            display_name: Some(display_name.into()),
            bio: None,
            avatar_url: None,
            banner_url: None,
            followers_count: 0,
            following_count: 0,
            notes_count: 0,
            is_locked: false,
            is_bot: false,
            is_cat: false,
            location: None,
            birthday: None,
            lang: None,
            fields: Vec::new(),
            followed_message: None,
            reaction_acceptance: None,
            created_at: None,
        }
    }

    pub fn name(&self) -> String {
        self.display_name
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| self.username.clone())
    }

    pub fn handle(&self) -> String {
        match &self.host {
            Some(host) if !host.is_empty() => format!("@{}@{}", self.username, host),
            _ => format!("@{}", self.username),
        }
    }

    pub fn route_handle(&self) -> String {
        match &self.host {
            Some(host) if !host.is_empty() => format!("{}@{}", self.username, host),
            _ => self.username.clone(),
        }
    }

    pub fn initials(&self) -> String {
        self.name()
            .chars()
            .filter(|c| !c.is_whitespace())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRelation {
    pub id: String,
    pub is_following: bool,
    pub is_followed: bool,
    pub is_blocking: bool,
    pub is_blocked: bool,
    pub is_muted: bool,
}
