use axum::{Extension, Json, extract::State};
use mithic_core::models::actor::{Actor, ActorId};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{
    get_actor_by_id, get_actor_by_username, is_following, is_blocking, is_muting,
    get_following, get_followers
};
use crate::services::relationship::{follow, unfollow, block, unblock, mute, unmute};
use crate::dto::actor_to_user;
use crate::state::AppState;
use serde::Deserialize;
use shared::{User, UserRelation};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserShowRequest {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub host: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListQueryRequest {
    pub user_id: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetUserRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsernameAvailableRequest {
    pub username: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableResponse {
    pub available: bool,
}

fn parse_actor_id(raw: &str) -> Result<ActorId> {
    raw.parse::<ActorId>()
        .map_err(|_| AppError::Validation("Invalid user id".to_string()))
}

pub async fn show(
    State(state): State<AppState>,
    Json(request): Json<UserShowRequest>,
) -> Result<Json<User>> {
    let actor = if let Some(id_str) = request.user_id {
        let actor_id = parse_actor_id(&id_str)?;
        get_actor_by_id(state.surreal(), &actor_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    } else if let Some(username) = request.username {
        get_actor_by_username(state.surreal(), &username)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    } else {
        return Err(AppError::Validation("userId or username required".to_string()));
    };

    let actor = actor.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    Ok(Json(actor_to_user(&actor)))
}

pub async fn relation(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<TargetUserRequest>,
) -> Result<Json<UserRelation>> {
    let my_id = auth.user_id;
    let target_id = parse_actor_id(&request.user_id)?;

    let is_following_val = is_following(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let is_followed_val = is_following(state.surreal(), &target_id, &my_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let is_blocking_val = is_blocking(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let is_blocked_val = is_blocking(state.surreal(), &target_id, &my_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let is_muted_val = is_muting(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(UserRelation {
        id: request.user_id,
        is_following: is_following_val,
        is_followed: is_followed_val,
        is_blocking: is_blocking_val,
        is_blocked: is_blocked_val,
        is_muted: is_muted_val,
    }))
}

pub async fn following(
    State(state): State<AppState>,
    Json(request): Json<UserListQueryRequest>,
) -> Result<Json<Vec<User>>> {
    let user_id = parse_actor_id(&request.user_id)?;
    let actors = get_following(state.surreal(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let users = actors.iter().map(actor_to_user).collect();
    Ok(Json(users))
}

pub async fn followers(
    State(state): State<AppState>,
    Json(request): Json<UserListQueryRequest>,
) -> Result<Json<Vec<User>>> {
    let user_id = parse_actor_id(&request.user_id)?;
    let actors = get_followers(state.surreal(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let users = actors.iter().map(actor_to_user).collect();
    Ok(Json(users))
}

pub async fn follow_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<TargetUserRequest>,
) -> Result<Json<UserRelation>> {
    let my_id = auth.user_id;
    let target_id = parse_actor_id(&request.user_id)?;

    if my_id == target_id {
        return Err(AppError::Validation("Cannot follow yourself".to_string()));
    }

    follow(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    relation(State(state), Extension(auth), Json(request)).await
}

pub async fn unfollow_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<TargetUserRequest>,
) -> Result<Json<UserRelation>> {
    let my_id = auth.user_id;
    let target_id = parse_actor_id(&request.user_id)?;

    unfollow(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    relation(State(state), Extension(auth), Json(request)).await
}

pub async fn block_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<TargetUserRequest>,
) -> Result<Json<UserRelation>> {
    let my_id = auth.user_id;
    let target_id = parse_actor_id(&request.user_id)?;

    block(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    relation(State(state), Extension(auth), Json(request)).await
}

pub async fn unblock_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<TargetUserRequest>,
) -> Result<Json<UserRelation>> {
    let my_id = auth.user_id;
    let target_id = parse_actor_id(&request.user_id)?;

    unblock(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    relation(State(state), Extension(auth), Json(request)).await
}

pub async fn mute_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<TargetUserRequest>,
) -> Result<Json<UserRelation>> {
    let my_id = auth.user_id;
    let target_id = parse_actor_id(&request.user_id)?;

    mute(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    relation(State(state), Extension(auth), Json(request)).await
}

pub async fn unmute_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<TargetUserRequest>,
) -> Result<Json<UserRelation>> {
    let my_id = auth.user_id;
    let target_id = parse_actor_id(&request.user_id)?;

    unmute(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    relation(State(state), Extension(auth), Json(request)).await
}

pub async fn search(
    State(state): State<AppState>,
    Json(request): Json<UserSearchRequest>,
) -> Result<Json<Vec<User>>> {
    let limit = request.limit.unwrap_or(20).min(100);

    let mut response = state.surreal()
        .query(
            "
            SELECT * FROM user 
            WHERE username_lower CONTAINS string::lowercase($query) 
               OR name CONTAINS $query
            LIMIT $limit;
            ",
        )
        .bind(("query", request.query))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<serde_json::Value> = response.take(0).map_err(|e| AppError::Internal(e.to_string()))?;
    let actors: Vec<Actor> = rows.into_iter()
        .map(|v| serde_json::from_value::<Actor>(v).map_err(|e| AppError::Internal(e.to_string())))
        .collect::<Result<Vec<Actor>>>()?;
    let users = actors.iter().map(actor_to_user).collect();
    Ok(Json(users))
}

pub async fn available(
    State(state): State<AppState>,
    Json(request): Json<UsernameAvailableRequest>,
) -> Result<Json<AvailableResponse>> {
    let actor = get_actor_by_username(state.surreal(), &request.username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(AvailableResponse {
        available: actor.is_none(),
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserNotesRequest {
    pub user_id: String,
    pub limit: Option<usize>,
}

pub async fn user_notes(
    State(state): State<AppState>,
    Json(request): Json<UserNotesRequest>,
) -> Result<Json<Vec<shared::Note>>> {
    let user_id = parse_actor_id(&request.user_id)?;
    let limit = request.limit.unwrap_or(20).min(100);

    let mut response = state.surreal()
        .query(
            "
            SELECT 
                *,
                actor_id.id AS actor_id,
                reply_id.id AS reply_id,
                renote_id.id AS renote_id
            FROM note
            WHERE actor_id = type::thing('user', $user)
            ORDER BY id DESC
            LIMIT $limit;
            ",
        )
        .bind(("user", user_id.to_string()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<serde_json::Value> = response.take(0).map_err(|e| AppError::Internal(e.to_string()))?;
    let notes: Vec<mithic_core::models::note::Note> = rows.into_iter()
        .map(|v| serde_json::from_value::<mithic_core::models::note::Note>(v).map_err(|e| AppError::Internal(e.to_string())))
        .collect::<Result<Vec<mithic_core::models::note::Note>>>()?;
    
    let mut note_dtos = Vec::new();
    for note in notes {
        let author = get_actor_by_id(state.surreal(), &note.actor_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;
        
        note_dtos.push(crate::dto::note_to_dto(&note, actor_to_user(&author)));
    }
    
    Ok(Json(note_dtos))
}
