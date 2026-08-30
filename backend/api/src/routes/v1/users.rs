//! Users / follow / block / mute / search / follow-requests

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use mithic_core::models::actor::{Actor, ActorId};
use mithic_core::models::notification::Notification;
use mithic_core::services::auth::{hash_password, verify_password};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::cache;
use mithic_db::queries::{
    get_actor_by_id, get_actor_by_username, get_followers, get_following, get_user_notes,
    is_blocking, is_following, is_muting,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared::{ProfileField, User, UserRelation};

use crate::dto::actor_to_user;
use crate::routes::v1::common::{
    normalize_handle, ok_null, parse_actor_id, parse_optional_note_id, rows_to_dtos, PagingQuery,
};
use crate::services::note::publish_notification;
use crate::services::relationship::{block, follow, mute, unblock, unfollow, unmute};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// me / profile
// ---------------------------------------------------------------------------

pub async fn me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<User>> {
    let actor = get_actor_by_id(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    Ok(Json(actor_to_user(&actor)))
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let t = s.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMeRequest {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    bio: Option<String>,
    #[serde(default)]
    is_locked: Option<bool>,
    #[serde(default)]
    is_bot: Option<bool>,
    #[serde(default)]
    is_cat: Option<bool>,
    #[serde(default)]
    avatar_url: Option<String>,
    #[serde(default)]
    banner_url: Option<String>,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    birthday: Option<String>,
    #[serde(default)]
    lang: Option<String>,
    #[serde(default)]
    fields: Option<Vec<ProfileField>>,
    #[serde(default)]
    followed_message: Option<String>,
    #[serde(default)]
    reaction_acceptance: Option<String>,
}

pub async fn update_me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<UpdateMeRequest>,
) -> Result<Json<User>> {
    let fields = request.fields.map(|rows| {
        rows.into_iter()
            .map(|f| ProfileField {
                name: f.name.trim().to_string(),
                value: f.value.trim().to_string(),
            })
            .filter(|f| !f.name.is_empty() && !f.value.is_empty())
            .take(16)
            .collect::<Vec<_>>()
    });
    let accept_set = request.reaction_acceptance.is_some();
    let reaction_acceptance = request.reaction_acceptance.and_then(|s| {
        let t = s.trim().to_string();
        match t.as_str() {
            "" | "all" => None,
            "likeOnly"
            | "likeOnlyForRemote"
            | "nonSensitiveOnly"
            | "nonSensitiveOnlyForLocalLikeOnlyForRemote" => Some(t),
            _ => None,
        }
    });

    state
        .surreal()
        .query(
            "UPDATE user SET
                name = IF $name != None THEN $name ELSE name END,
                bio = IF $bio != None THEN $bio ELSE bio END,
                is_locked = IF $is_locked != None THEN $is_locked ELSE is_locked END,
                is_bot = IF $is_bot != None THEN $is_bot ELSE is_bot END,
                is_cat = IF $is_cat != None THEN $is_cat ELSE is_cat END,
                avatar_url = IF $avatar_set THEN $avatar_url ELSE avatar_url END,
                banner_url = IF $banner_set THEN $banner_url ELSE banner_url END,
                location = IF $location_set THEN $location ELSE location END,
                birthday = IF $birthday_set THEN $birthday ELSE birthday END,
                lang = IF $lang_set THEN $lang ELSE lang END,
                fields = IF $fields != None THEN $fields ELSE fields END,
                followed_message = IF $followed_set THEN $followed_message ELSE followed_message END,
                reaction_acceptance = IF $accept_set THEN $reaction_acceptance ELSE reaction_acceptance END,
                updated_at = time::now()
             WHERE id = type::record('user', $id);",
        )
        .bind(("id", auth.user_id.to_string()))
        .bind(("name", empty_to_none(request.display_name)))
        .bind(("bio", request.bio.clone().map(|s| s.trim().to_string())))
        .bind(("is_locked", request.is_locked))
        .bind(("is_bot", request.is_bot))
        .bind(("is_cat", request.is_cat))
        .bind(("avatar_set", request.avatar_url.is_some()))
        .bind(("avatar_url", empty_to_none(request.avatar_url)))
        .bind(("banner_set", request.banner_url.is_some()))
        .bind(("banner_url", empty_to_none(request.banner_url)))
        .bind(("location_set", request.location.is_some()))
        .bind(("location", empty_to_none(request.location)))
        .bind(("birthday_set", request.birthday.is_some()))
        .bind(("birthday", empty_to_none(request.birthday)))
        .bind(("lang_set", request.lang.is_some()))
        .bind(("lang", empty_to_none(request.lang)))
        .bind(("fields", fields))
        .bind(("followed_set", request.followed_message.is_some()))
        .bind(("followed_message", empty_to_none(request.followed_message)))
        .bind(("accept_set", accept_set))
        .bind(("reaction_acceptance", reaction_acceptance))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    me(State(state), Extension(auth)).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<Value>> {
    let actor = get_actor_by_id(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let hash = actor
        .password_hash
        .as_ref()
        .ok_or_else(|| AppError::Validation("No password set".to_string()))?;

    if !verify_password(&request.current_password, hash)? {
        return Err(AppError::Unauthorized(
            "Invalid current password".to_string(),
        ));
    }

    if request.new_password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let new_hash = hash_password(&request.new_password)?;
    state
        .surreal()
        .query(
            "UPDATE user SET password_hash = $hash WHERE id = type::record('user', $user_id);",
        )
        .bind(("user_id", auth.user_id.to_string()))
        .bind(("hash", new_hash))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ok_null())
}

#[derive(Debug, Deserialize)]
pub struct CheckHandleQuery {
    handle: String,
}

#[derive(Debug, Serialize)]
pub struct HandleAvailability {
    available: bool,
}

pub async fn check_handle(
    State(state): State<AppState>,
    Query(query): Query<CheckHandleQuery>,
) -> Result<Json<HandleAvailability>> {
    let username = normalize_handle(&query.handle);
    let actor = get_actor_by_username(state.surreal(), &username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(HandleAvailability {
        available: actor.is_none() && username.len() >= 3,
    }))
}

pub async fn show_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<User>> {
    let actor = get_actor_by_username(state.surreal(), &normalize_handle(&username))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    Ok(Json(actor_to_user(&actor)))
}

pub async fn user_notes(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(paging): Query<PagingQuery>,
) -> Result<Json<Vec<shared::Note>>> {
    let actor = get_actor_by_username(state.surreal(), &normalize_handle(&username))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let notes = get_user_notes(
        state.surreal(),
        &actor.id,
        paging.limit.unwrap_or(20).min(100),
        parse_optional_note_id(&paging.until_id)?,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(rows_to_dtos(&state, notes).await))
}

// ---------------------------------------------------------------------------
// relation / follow / block / mute
// ---------------------------------------------------------------------------

async fn check_blocking_cached(
    state: &AppState,
    blocker_id: &ActorId,
    blocked_id: &ActorId,
) -> anyhow::Result<bool> {
    if cache::block_set_contains(
        state.dragonfly(),
        &blocker_id.to_string(),
        &blocked_id.to_string(),
    )
    .await
    {
        return Ok(true);
    }
    is_blocking(state.surreal(), blocker_id, blocked_id).await
}

async fn check_muting_cached(
    state: &AppState,
    muter_id: &ActorId,
    muted_id: &ActorId,
) -> anyhow::Result<bool> {
    if cache::mute_set_contains(
        state.dragonfly(),
        &muter_id.to_string(),
        &muted_id.to_string(),
    )
    .await
    {
        return Ok(true);
    }
    is_muting(state.surreal(), muter_id, muted_id).await
}

async fn build_relation(state: &AppState, my_id: ActorId, target_id: ActorId) -> Result<UserRelation> {
    let is_following_val = is_following(state.surreal(), &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let is_followed_val = is_following(state.surreal(), &target_id, &my_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let is_blocking_val = check_blocking_cached(state, &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let is_blocked_val = check_blocking_cached(state, &target_id, &my_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let is_muted_val = check_muting_cached(state, &my_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(UserRelation {
        id: target_id.to_string(),
        is_following: is_following_val,
        is_followed: is_followed_val,
        is_blocking: is_blocking_val,
        is_blocked: is_blocked_val,
        is_muted: is_muted_val,
    })
}

pub async fn relation(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    Ok(Json(
        build_relation(&state, auth.user_id, target_id).await?,
    ))
}

pub async fn following(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<User>>> {
    let user_id = parse_actor_id(&id)?;
    let actors = get_following(state.surreal(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(actors.iter().map(actor_to_user).collect()))
}

pub async fn followers(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<User>>> {
    let user_id = parse_actor_id(&id)?;
    let actors = get_followers(state.surreal(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(actors.iter().map(actor_to_user).collect()))
}

pub async fn follow_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let target_id = parse_actor_id(&id)?;
    if target_id == auth.user_id {
        return Err(AppError::Validation("Cannot follow yourself".to_string()));
    }

    let already = is_following(state.surreal(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if !already {
        follow(state.surreal(), &auth.user_id, &target_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let sender = get_actor_by_id(state.surreal(), &auth.user_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let notif = Notification::follow(target_id, auth.user_id);
        publish_notification(&state, &notif, sender.as_ref(), None).await;
    }
    let followed_message = get_actor_by_id(state.surreal(), &target_id)
        .await
        .ok()
        .flatten()
        .and_then(|a| a.followed_message);
    Ok(Json(serde_json::json!({ "followedMessage": followed_message })))
}

pub async fn unfollow_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let target_id = parse_actor_id(&id)?;
    unfollow(state.surreal(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

pub async fn block_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    block(state.surreal(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::block_set_add(
        state.dragonfly(),
        &auth.user_id.to_string(),
        &target_id.to_string(),
        cache::BLOCK_MUTE_SET_TTL,
    )
    .await;
    Ok(Json(
        build_relation(&state, auth.user_id, target_id).await?,
    ))
}

pub async fn unblock_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    unblock(state.surreal(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::block_set_remove(
        state.dragonfly(),
        &auth.user_id.to_string(),
        &target_id.to_string(),
    )
    .await;
    Ok(Json(
        build_relation(&state, auth.user_id, target_id).await?,
    ))
}

pub async fn mute_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    mute(state.surreal(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::mute_set_add(
        state.dragonfly(),
        &auth.user_id.to_string(),
        &target_id.to_string(),
        cache::BLOCK_MUTE_SET_TTL,
    )
    .await;
    Ok(Json(
        build_relation(&state, auth.user_id, target_id).await?,
    ))
}

pub async fn unmute_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    unmute(state.surreal(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::mute_set_remove(
        state.dragonfly(),
        &auth.user_id.to_string(),
        &target_id.to_string(),
    )
    .await;
    Ok(Json(
        build_relation(&state, auth.user_id, target_id).await?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
}

async fn actors_from_edge(
    state: &AppState,
    table: &str,
    user_id: &ActorId,
    limit: usize,
) -> Result<Vec<User>> {
    let query = format!(
        "SELECT out.* AS actor FROM {table} WHERE in = type::record('user', $user) LIMIT $limit;"
    );
    let mut response = state
        .surreal()
        .query(&query)
        .bind(("user", user_id.to_string()))
        .bind(("limit", limit as i64))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<surrealdb::types::Value> = response
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut users = Vec::new();
    for val in rows {
        let mut json = val.into_json_value();
        mithic_db::queries::strip_record_prefixes(&mut json);
        if let Some(actor_val) = json.get("actor").cloned()
            && let Ok(actor) = serde_json::from_value::<Actor>(actor_val)
        {
            users.push(actor_to_user(&actor));
        }
    }
    Ok(users)
}

pub async fn list_blocks(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<User>>> {
    let limit = q.limit.unwrap_or(20).min(100);
    Ok(Json(
        actors_from_edge(&state, "block", &auth.user_id, limit).await?,
    ))
}

pub async fn list_mutes(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<User>>> {
    let limit = q.limit.unwrap_or(20).min(100);
    Ok(Json(
        actors_from_edge(&state, "mute", &auth.user_id, limit).await?,
    ))
}

// ---------------------------------------------------------------------------
// search / suggested
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SuggestedQuery {
    pub limit: Option<usize>,
}

fn actors_from_rows(rows: Vec<surrealdb::types::Value>) -> Result<Vec<Actor>> {
    rows.into_iter()
        .map(|v| {
            let mut json = v.into_json_value();
            mithic_db::queries::strip_record_prefixes(&mut json);
            serde_json::from_value::<Actor>(json).map_err(|e| AppError::Internal(e.to_string()))
        })
        .collect()
}

pub async fn search_users(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<User>>> {
    let limit = query.limit.unwrap_or(20).min(100);
    let mut response = state
        .surreal()
        .query(
            "
            SELECT * FROM user
            WHERE username_lower CONTAINS string::lowercase($query)
               OR name CONTAINS $query
            LIMIT $limit;
            ",
        )
        .bind(("query", query.q))
        .bind(("limit", limit as i64))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<surrealdb::types::Value> = response
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let actors = actors_from_rows(rows)?;
    Ok(Json(actors.iter().map(actor_to_user).collect()))
}

/// ローカル人気ユーザー（followers_count 降順）。おすすめ枠用の最小実装。
pub async fn suggested_users(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Query(query): Query<SuggestedQuery>,
) -> Result<Json<Vec<User>>> {
    let limit = query.limit.unwrap_or(5).min(20);
    let exclude = auth.map(|Extension(a)| a.user_id.to_string());

    // ローカル優先は FE 側で host フィルタ。DB は人気順の最小クエリのみ。
    let mut response = if let Some(ref me) = exclude {
        state
            .surreal()
            .query(
                "
                SELECT * FROM user
                WHERE id != type::record('user', $me)
                ORDER BY followers_count DESC, notes_count DESC
                LIMIT $limit;
                ",
            )
            .bind(("me", me.clone()))
            .bind(("limit", (limit * 2) as i64))
            .await
    } else {
        state
            .surreal()
            .query(
                "
                SELECT * FROM user
                ORDER BY followers_count DESC, notes_count DESC
                LIMIT $limit;
                ",
            )
            .bind(("limit", (limit * 2) as i64))
            .await
    }
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<surrealdb::types::Value> = response
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let actors = actors_from_rows(rows)?;
    let users: Vec<User> = actors
        .iter()
        .filter(|a| a.host.as_ref().map(|h| h.is_empty()).unwrap_or(true))
        .take(limit)
        .map(actor_to_user)
        .collect();
    Ok(Json(users))
}

// ---------------------------------------------------------------------------
// follow requests
// ---------------------------------------------------------------------------

pub async fn list_follow_requests(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<User>>> {
    let mut response = state
        .surreal()
        .query(
            "
            SELECT out.* AS actor FROM follow
            WHERE in = type::record('user', $user) AND is_accepted = false;
            ",
        )
        .bind(("user", auth.user_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<surrealdb::types::Value> = response
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut users = Vec::new();
    for val in rows {
        let mut json = val.into_json_value();
        mithic_db::queries::strip_record_prefixes(&mut json);
        if let Some(actor_val) = json.get("actor").cloned()
            && let Ok(actor) = serde_json::from_value::<Actor>(actor_val)
        {
            users.push(actor_to_user(&actor));
        }
    }
    Ok(Json(users))
}

pub async fn accept_follow_request(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    state
        .surreal()
        .query(
            "
            UPDATE follow SET is_accepted = true
            WHERE in = type::record('user', $me) AND out = type::record('user', $target);
            UPDATE user SET following_count = <int>(following_count OR 0) + 1
            WHERE id = type::record('user', $target);
            ",
        )
        .bind(("me", auth.user_id.to_string()))
        .bind(("target", target_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let notif = Notification::new(
        mithic_core::models::notification::NotificationType::FollowRequestAccepted,
        target_id,
        Some(auth.user_id),
        None,
    );
    publish_notification(&state, &notif, None, None).await;

    Ok(Json(
        build_relation(&state, auth.user_id, target_id).await?,
    ))
}

pub async fn reject_follow_request(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let target_id = parse_actor_id(&id)?;
    state
        .surreal()
        .query(
            "
            DELETE follow
            WHERE in = type::record('user', $me)
              AND out = type::record('user', $target)
              AND is_accepted = false;
            ",
        )
        .bind(("me", auth.user_id.to_string()))
        .bind(("target", target_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn cancel_follow_request(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let target_id = parse_actor_id(&id)?;
    unfollow(state.surreal(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}
