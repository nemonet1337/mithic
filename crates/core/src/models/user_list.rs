use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

pub type UserListId = Ulid;
pub type UserListMembershipId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserList {
    pub id: UserListId,
    pub user_id: ActorId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl UserList {
    pub fn new(user_id: ActorId, name: String) -> Self {
        Self { id: UserListId::new(), user_id, name, created_at: Utc::now(), updated_at: None }
    }

    pub fn rename(&mut self, name: String) { self.name = name; self.updated_at = Some(Utc::now()); }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListMembership {
    pub id: UserListMembershipId,
    pub list_id: UserListId,
    pub user_id: ActorId,
    pub created_at: DateTime<Utc>,
}

impl UserListMembership {
    pub fn new(list_id: UserListId, user_id: ActorId) -> Self {
        Self { id: UserListMembershipId::new(), list_id, user_id, created_at: Utc::now() }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserListRequest { pub name: String }

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserListRequest { pub name: String }

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
        Self { id: list.id.to_string(), name: list.name, created_at: list.created_at, updated_at: list.updated_at }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListMembershipResponse {
    pub id: String,
    pub list_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
}

impl From<UserListMembership> for UserListMembershipResponse {
    fn from(m: UserListMembership) -> Self {
        Self { id: m.id.to_string(), list_id: m.list_id.to_string(), user_id: m.user_id.to_string(), created_at: m.created_at }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddUserToListRequest { pub user_id: String }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListWithMembers {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
