//! User List API endpoints
//!
//! Provides API for managing user lists.

use axum::{
    extract::{Path, State},
    Json,
};
use tracing::{info, warn};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{CreateUserListRequest, UpdateUserListRequest, AddUserToListRequest, UserList, UserListMembership, UserListResponse, UserListMembershipResponse, UserListWithMembers, UserListId},
    state::{AppState, AuthUser},
};

/// List user's lists
///
/// Returns all lists owned by the authenticated user.
pub async fn get_user_lists(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserListResponse>>> {
    let user_id = auth_user.user_id;
    
    let lists: Vec<UserList> = state.surreal()
        .query("SELECT * FROM user_list WHERE user_id = $user_id ORDER BY created_at DESC")
        .bind(("user_id", user_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();
    
    let responses: Vec<UserListResponse> = lists
        .into_iter()
        .map(|l| l.into())
        .collect();
    
    Ok(Json(responses))
}

/// Create a new list
///
/// Creates a new user list with the given name.
pub async fn create_list(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateUserListRequest>,
) -> Result<Json<UserListResponse>> {
    let user_id = auth_user.user_id;
    
    if request.name.is_empty() {
        return Err(AppError::Validation("List name cannot be empty".to_string()));
    }
    
    let list = UserList::new(user_id, request.name);
    
    state.surreal()
        .create::<Option<UserList>>(
            ("user_list", list.id.to_string()),
        )
        .content(list.clone())
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} created list: {}", auth_user.username, list.name);
    
    Ok(Json(list.into()))
}

/// Get a specific list
///
/// Returns a single list with its members.
pub async fn get_list(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserListWithMembers>> {
    let user_id = auth_user.user_id;
    let list_id = id.parse::<UserListId>()
        .map_err(|_| AppError::Validation("Invalid list ID".to_string()))?;
    
    let list: UserList = state.surreal()
        .select(("user_list", list_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("List not found".to_string()))?;
    
    // Check ownership
    if list.user_id != user_id {
        return Err(AppError::Forbidden("You don't own this list".to_string()));
    }
    
    // Get members
    let memberships: Vec<UserListMembership> = state.surreal()
        .query("SELECT * FROM user_list_membership WHERE list_id = $list_id")
        .bind(("list_id", list_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();
    
    let member_ids: Vec<String> = memberships
        .into_iter()
        .map(|m| m.user_id.to_string())
        .collect();
    
    let response = UserListWithMembers {
        id: list.id.to_string(),
        name: list.name,
        members: member_ids,
        created_at: list.created_at,
        updated_at: list.updated_at,
    };
    
    Ok(Json(response))
}

/// Update a list
///
/// Updates the name of an existing list.
pub async fn update_list(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateUserListRequest>,
) -> Result<Json<UserListResponse>> {
    let user_id = auth_user.user_id;
    let list_id = id.parse::<UserListId>()
        .map_err(|_| AppError::Validation("Invalid list ID".to_string()))?;
    
    if request.name.is_empty() {
        return Err(AppError::Validation("List name cannot be empty".to_string()));
    }
    
    let mut list: UserList = state.surreal()
        .select(("user_list", list_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("List not found".to_string()))?;
    
    // Check ownership
    if list.user_id != user_id {
        return Err(AppError::Forbidden("You don't own this list".to_string()));
    }
    
    list.rename(request.name);
    
    state.surreal()
        .update::<Option<UserList>>(
            ("user_list", list_id.to_string()),
        )
        .merge(list.clone())
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} updated list: {}", auth_user.username, list.name);
    
    Ok(Json(list.into()))
}

/// Delete a list
///
/// Deletes a list and all its memberships.
pub async fn delete_list(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let user_id = auth_user.user_id;
    let list_id = id.parse::<UserListId>()
        .map_err(|_| AppError::Validation("Invalid list ID".to_string()))?;
    
    let list: UserList = state.surreal()
        .select(("user_list", list_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("List not found".to_string()))?;
    
    // Check ownership
    if list.user_id != user_id {
        return Err(AppError::Forbidden("You don't own this list".to_string()));
    }
    
    // Delete memberships first
    state.surreal()
        .query("DELETE user_list_membership WHERE list_id = $list_id")
        .bind(("list_id", list_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;
    
    // Delete the list
    state.surreal()
        .query("DELETE user_list WHERE id = $id")
        .bind(("id", format!("user_list:{}", list_id)))
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} deleted list: {}", auth_user.username, list.name);
    
    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// Add user to list
///
/// Adds a user to the specified list.
pub async fn add_user_to_list(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    Json(request): Json<AddUserToListRequest>,
) -> Result<Json<UserListMembershipResponse>> {
    let user_id = auth_user.user_id;
    let list_id = list_id.parse::<UserListId>()
        .map_err(|_| AppError::Validation("Invalid list ID".to_string()))?;
    
    let target_user_id = request.user_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;
    
    let list: UserList = state.surreal()
        .select(("user_list", list_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("List not found".to_string()))?;
    
    // Check ownership
    if list.user_id != user_id {
        return Err(AppError::Forbidden("You don't own this list".to_string()));
    }
    
    // Check if already a member
    let existing: Option<UserListMembership> = state.surreal()
        .query("SELECT * FROM user_list_membership WHERE list_id = $list_id AND user_id = $user_id")
        .bind(("list_id", list_id.to_string()))
        .bind(("user_id", target_user_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default()
        .into_iter()
        .next();
    
    if existing.is_some() {
        return Err(AppError::Conflict("User is already in this list".to_string()));
    }
    
    let membership = UserListMembership::new(list_id, target_user_id);
    
    state.surreal()
        .create::<Option<UserListMembership>>(
            ("user_list_membership", membership.id.to_string()),
        )
        .content(membership.clone())
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} added user {} to list {}", auth_user.username, target_user_id, list_id);
    
    Ok(Json(membership.into()))
}

/// Remove user from list
///
/// Removes a user from the specified list.
pub async fn remove_user_from_list(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path((list_id, member_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let user_id = auth_user.user_id;
    let list_id = list_id.parse::<UserListId>()
        .map_err(|_| AppError::Validation("Invalid list ID".to_string()))?;
    
    let target_user_id = member_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;
    
    let list: UserList = state.surreal()
        .select(("user_list", list_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("List not found".to_string()))?;
    
    // Check ownership
    if list.user_id != user_id {
        return Err(AppError::Forbidden("You don't own this list".to_string()));
    }
    
    state.surreal()
        .query("DELETE user_list_membership WHERE list_id = $list_id AND user_id = $user_id")
        .bind(("list_id", list_id.to_string()))
        .bind(("user_id", target_user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} removed user {} from list {}", auth_user.username, target_user_id, list_id);
    
    Ok(Json(serde_json::json!({
        "success": true
    })))
}
