//! Hashtag model
//!
//! Represents a hashtag with usage statistics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

/// Hashtag ID
pub type HashtagId = Ulid;

/// Hashtag model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hashtag {
    pub id: HashtagId,

    /// Hashtag name (lowercase, normalized)
    pub name: String,

    /// Total usage count (number of notes with this tag)
    pub count: i32,

    /// User IDs who have used this hashtag (for unique count)
    pub mentioned_user_ids: Vec<ActorId>,

    /// Number of unique users who used this hashtag
    pub mentioned_users_count: i32,

    /// Local user IDs who used this hashtag
    pub mentioned_local_user_ids: Vec<ActorId>,

    /// Number of unique local users
    pub mentioned_local_users_count: i32,

    /// Remote user IDs who used this hashtag
    pub mentioned_remote_user_ids: Vec<ActorId>,

    /// Number of unique remote users
    pub mentioned_remote_users_count: i32,

    /// When the hashtag was first created
    pub created_at: DateTime<Utc>,

    /// When the hashtag was last updated
    pub updated_at: DateTime<Utc>,
}

impl Hashtag {
    /// Create a new hashtag
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: HashtagId::new(),
            name: name.into().to_lowercase(),
            count: 0,
            mentioned_user_ids: Vec::new(),
            mentioned_users_count: 0,
            mentioned_local_user_ids: Vec::new(),
            mentioned_local_users_count: 0,
            mentioned_remote_user_ids: Vec::new(),
            mentioned_remote_users_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update count and user tracking
    pub fn update_usage(&mut self, user_id: ActorId, is_local: bool) {
        self.count += 1;
        self.updated_at = Utc::now();

        // Track unique users
        if !self.mentioned_user_ids.contains(&user_id) {
            self.mentioned_user_ids.push(user_id.clone());
            self.mentioned_users_count += 1;

            if is_local {
                self.mentioned_local_user_ids.push(user_id.clone());
                self.mentioned_local_users_count += 1;
            } else {
                self.mentioned_remote_user_ids.push(user_id.clone());
                self.mentioned_remote_users_count += 1;
            }
        }
    }
}

/// Trending hashtag response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingHashtag {
    pub tag: String,
    pub chart: Vec<i32>,
    pub users_count: i32,
}

/// Hashtag search query
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashtagSearchQuery {
    pub q: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl HashtagSearchQuery {
    pub fn limit(&self) -> i32 {
        self.limit.unwrap_or(20).min(100)
    }

    pub fn offset(&self) -> i32 {
        self.offset.unwrap_or(0)
    }
}

/// Hashtag timeline query
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashtagTimelineQuery {
    pub tag: String,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
    pub limit: Option<i32>,
    pub with_files: Option<bool>,
}

impl HashtagTimelineQuery {
    pub fn limit(&self) -> i32 {
        self.limit.unwrap_or(10).min(30)
    }
}
