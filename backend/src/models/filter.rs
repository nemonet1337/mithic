//! Filter models
//!
//! Post content filtering (word mute, regex filters).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

/// Filter ID
pub type FilterId = Ulid;

/// Content filter (word mute)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub id: FilterId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    
    /// User who created the filter
    pub user_id: Ulid,
    
    #[validate(length(min = 1, max = 256))]
    pub phrase: String,
    
    /// Whether this is a regex pattern
    pub is_regex: bool,
    
    /// Whether to hide completely or just collapse
    pub hide_completely: bool,
    
    /// Expiration time (optional)
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Context where filter applies (home, notifications, public, thread)
    pub context: Vec<String>,
}

/// Filter creation request
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFilterRequest {
    #[validate(length(min = 1, max = 256))]
    pub phrase: String,
    
    pub is_regex: Option<bool>,
    pub hide_completely: Option<bool>,
    pub expires_in: Option<i64>, // seconds
    pub context: Option<Vec<String>>,
}

/// Filter update request
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFilterRequest {
    #[validate(length(min = 1, max = 256))]
    pub phrase: Option<String>,
    pub is_regex: Option<bool>,
    pub hide_completely: Option<bool>,
    pub expires_in: Option<i64>,
    pub context: Option<Vec<String>>,
}

/// Filter response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterResponse {
    pub id: FilterId,
    pub phrase: String,
    pub is_regex: bool,
    pub hide_completely: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub context: Vec<String>,
}

impl From<Filter> for FilterResponse {
    fn from(filter: Filter) -> Self {
        Self {
            id: filter.id,
            phrase: filter.phrase,
            is_regex: filter.is_regex,
            hide_completely: filter.hide_completely,
            expires_at: filter.expires_at,
            context: filter.context,
        }
    }
}
