//! User List model
//!
//! Stores user-created lists for organizing followed users.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

/// User List ID
pub type UserListId = Ulid;

/// User List membership ID
pub type UserListMembershipId = Ulid;

/// User-created list for organizing followed users
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserList {
    pub id: UserListId,

    /// Owner user ID
    pub user_id: ActorId,

    /// List name
    pub name: String,

    /// When the list was created
    pub created_at: DateTime<Utc>,

    /// When the list was last updated
    pub updated_at: Option<DateTime<Utc>>,
}

impl UserList {
    /// Create a new user list
    pub fn new(user_id: ActorId, name: String) -> Self {
        Self {
            id: UserListId::new(),
            user_id,
            name,
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    /// Update the list name
    pub fn rename(&mut self, name: String) {
        self.name = name;
        self.updated_at = Some(Utc::now());
    }
}

/// User list membership (user in a list)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListMembership {
    pub id: UserListMembershipId,

    /// List ID
    pub list_id: UserListId,

    /// User ID
    pub user_id: ActorId,

    /// When the user was added to the list
    pub created_at: DateTime<Utc>,
}

impl UserListMembership {
    /// Create a new membership
    pub fn new(list_id: UserListId, user_id: ActorId) -> Self {
        Self {
            id: UserListMembershipId::new(),
            list_id,
            user_id,
            created_at: Utc::now(),
        }
    }
}

/// Create user list request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserListRequest {
    pub name: String,
}

/// Update user list request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserListRequest {
    pub name: String,
}

/// User list response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListResponse {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<UserList> for UserListResponse {
    fn from(list: UserList) -> Self {
        Self {
            id: list.id.to_string(),
            name: list.name,
            created_at: list.created_at,
            updated_at: list.updated_at,
        }
    }
}

/// User list membership response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListMembershipResponse {
    pub id: String,
    pub list_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
}

impl From<UserListMembership> for UserListMembershipResponse {
    fn from(membership: UserListMembership) -> Self {
        Self {
            id: membership.id.to_string(),
            list_id: membership.list_id.to_string(),
            user_id: membership.user_id.to_string(),
            created_at: membership.created_at,
        }
    }
}

/// Add user to list request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddUserToListRequest {
    pub user_id: String,
}

/// User list with members
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListWithMembers {
    pub id: String,
    pub name: String,
    pub members: Vec<String>, // User IDs
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
