//! Users / follow / block / mute / search / follow-requests

use crate::auth::{hash_password, verify_password};
use crate::db::cache;
use crate::db::queries::actors::{ACTOR_COLUMNS, ActorRow};
use crate::db::queries::{
    block_user as block, follow_request_state, follow_user as follow, get_actor_by_id,
    get_actor_by_username, get_followers, get_following, get_user_notes, is_blocking, is_following,
    is_muting, mute_user as mute, unblock_user as unblock, unfollow_user as unfollow,
    unmute_user as unmute,
};
use crate::models::actor::{Actor, ActorId};
use crate::models::notification::Notification;
use crate::{AppError, AuthUser, Result};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared::{ProfileField, UpdateProfileRequest, User, UserRelation};

use crate::dto::actor_to_user;
use crate::routes::v1::common::{
    PagingQuery, normalize_handle, ok_null, parse_actor_id, parse_optional_note_id, rows_to_dtos,
};
use crate::services::note::publish_notification;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// me / profile
// ---------------------------------------------------------------------------

pub async fn me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<User>> {
    let actor = get_actor_by_id(state.db(), &auth.user_id)
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

pub async fn update_me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<UpdateProfileRequest>,
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

    let fields_json = fields.as_ref().and_then(|f| serde_json::to_value(f).ok());

    // 未送信フィールドは COALESCE で現在値を維持する (`IF $x != None` の置換)
    sqlx::query(
        "UPDATE users SET
            name = COALESCE($2, name),
            bio = COALESCE($3, bio),
            is_locked = COALESCE($4, is_locked),
            is_bot = COALESCE($5, is_bot),
            is_cat = COALESCE($6, is_cat),
            avatar_url = COALESCE($7, avatar_url),
            banner_url = COALESCE($8, banner_url),
            location = COALESCE($9, location),
            birthday = COALESCE($10, birthday),
            lang = COALESCE($11, lang),
            fields = COALESCE($12, fields),
            followed_message = COALESCE($13, followed_message),
            reaction_acceptance = COALESCE($14, reaction_acceptance),
            updated_at = now()
         WHERE id = $1",
    )
    .bind(auth.user_id.to_string())
    .bind(empty_to_none(request.display_name))
    .bind(request.bio.map(|s| s.trim().to_string()))
    .bind(request.is_locked)
    .bind(request.is_bot)
    .bind(request.is_cat)
    .bind(empty_to_none(request.avatar_url))
    .bind(empty_to_none(request.banner_url))
    .bind(empty_to_none(request.location))
    .bind(empty_to_none(request.birthday))
    .bind(empty_to_none(request.lang))
    .bind(fields_json)
    .bind(empty_to_none(request.followed_message))
    .bind(reaction_acceptance)
    .execute(state.db())
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
    let actor = get_actor_by_id(state.db(), &auth.user_id)
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

    if request.new_password == request.current_password {
        return Err(AppError::Validation(
            "New password must be different from the current one".to_string(),
        ));
    }

    let new_hash = hash_password(&request.new_password)?;
    sqlx::query("UPDATE users SET password_hash = $2 WHERE id = $1")
        .bind(auth.user_id.to_string())
        .bind(new_hash)
        .execute(state.db())
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
    let actor = get_actor_by_username(state.db(), &username)
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
    let actor = get_actor_by_username(state.db(), &normalize_handle(&username))
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
    let actor = get_actor_by_username(state.db(), &normalize_handle(&username))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let notes = get_user_notes(
        state.db(),
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
        state.redis(),
        &blocker_id.to_string(),
        &blocked_id.to_string(),
    )
    .await
    {
        return Ok(true);
    }
    is_blocking(state.db(), blocker_id, blocked_id).await
}

async fn check_muting_cached(
    state: &AppState,
    muter_id: &ActorId,
    muted_id: &ActorId,
) -> anyhow::Result<bool> {
    if cache::mute_set_contains(state.redis(), &muter_id.to_string(), &muted_id.to_string()).await {
        return Ok(true);
    }
    is_muting(state.db(), muter_id, muted_id).await
}

async fn build_relation(
    state: &AppState,
    my_id: ActorId,
    target_id: ActorId,
) -> Result<UserRelation> {
    let (mine, is_followed_val, is_blocking_val, is_blocked_val, is_muted_val) = tokio::join!(
        follow_request_state(state.db(), &my_id, &target_id),
        is_following(state.db(), &target_id, &my_id),
        check_blocking_cached(state, &my_id, &target_id),
        check_blocking_cached(state, &target_id, &my_id),
        check_muting_cached(state, &my_id, &target_id),
    );
    let mine = mine.map_err(|e| AppError::Internal(e.to_string()))?;
    let is_followed_val = is_followed_val.map_err(|e| AppError::Internal(e.to_string()))?;
    let is_blocking_val = is_blocking_val.map_err(|e| AppError::Internal(e.to_string()))?;
    let is_blocked_val = is_blocked_val.map_err(|e| AppError::Internal(e.to_string()))?;
    let is_muted_val = is_muted_val.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(UserRelation {
        id: target_id.to_string(),
        is_following: mine == Some(true),
        is_followed: is_followed_val,
        is_blocking: is_blocking_val,
        is_blocked: is_blocked_val,
        is_muted: is_muted_val,
        is_follow_requested: mine == Some(false),
    })
}

pub async fn relation(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    Ok(Json(build_relation(&state, auth.user_id, target_id).await?))
}

pub async fn following(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<User>>> {
    let user_id = parse_actor_id(&id)?;
    let actors = get_following(state.db(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(actors.iter().map(actor_to_user).collect()))
}

pub async fn followers(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<User>>> {
    let user_id = parse_actor_id(&id)?;
    let actors = get_followers(state.db(), &user_id)
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

    let target = get_actor_by_id(state.db(), &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let existing = follow_request_state(state.db(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let accepted = !target.is_locked;
    if existing != Some(true) {
        follow(state.db(), &auth.user_id, &target_id, accepted)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let sender = get_actor_by_id(state.db(), &auth.user_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let notif = if accepted {
            Notification::follow(target_id, auth.user_id)
        } else {
            Notification::new(
                crate::models::notification::NotificationType::FollowRequest,
                target_id,
                Some(auth.user_id),
                None,
            )
        };
        publish_notification(&state, &notif, sender.as_ref(), None).await;
    }

    Ok(Json(serde_json::json!({
        "followedMessage": target.followed_message,
        "isPending": existing != Some(true) && !accepted,
    })))
}

pub async fn unfollow_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let target_id = parse_actor_id(&id)?;
    unfollow(state.db(), &auth.user_id, &target_id)
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
    block(state.db(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::block_set_add(
        state.redis(),
        &auth.user_id.to_string(),
        &target_id.to_string(),
        cache::BLOCK_MUTE_SET_TTL,
    )
    .await;
    Ok(Json(build_relation(&state, auth.user_id, target_id).await?))
}

pub async fn unblock_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    unblock(state.db(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::block_set_remove(
        state.redis(),
        &auth.user_id.to_string(),
        &target_id.to_string(),
    )
    .await;
    Ok(Json(build_relation(&state, auth.user_id, target_id).await?))
}

pub async fn mute_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    mute(state.db(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::mute_set_add(
        state.redis(),
        &auth.user_id.to_string(),
        &target_id.to_string(),
        cache::BLOCK_MUTE_SET_TTL,
    )
    .await;
    Ok(Json(build_relation(&state, auth.user_id, target_id).await?))
}

pub async fn unmute_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    unmute(state.db(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = cache::mute_set_remove(
        state.redis(),
        &auth.user_id.to_string(),
        &target_id.to_string(),
    )
    .await;
    Ok(Json(build_relation(&state, auth.user_id, target_id).await?))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
}

/// block / mute の edge から相手アクターを取得する。
/// テーブル名は match で分岐し、SQL インジェクションを不能にする。
async fn actors_from_edge(
    state: &AppState,
    table: &str,
    user_id: &ActorId,
    limit: usize,
) -> Result<Vec<User>> {
    let sql = match table {
        "block" => {
            "SELECT u.* FROM users u JOIN blocks b ON u.id = b.blocked_id
             WHERE b.blocker_id = $1 LIMIT $2"
        }
        "mute" => {
            "SELECT u.* FROM users u JOIN mutes m ON u.id = m.muted_id
             WHERE m.muter_id = $1 LIMIT $2"
        }
        other => {
            return Err(AppError::Validation(format!("Unknown relation: {other}")));
        }
    };

    let rows = sqlx::query_as::<_, crate::db::queries::actors::ActorRow>(sql)
        .bind(user_id.to_string())
        .bind(limit as i64)
        .fetch_all(state.db())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_into().ok())
        .map(|actor| actor_to_user(&actor))
        .collect())
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

fn actors_from_rows(rows: Vec<ActorRow>) -> Result<Vec<Actor>> {
    rows.into_iter()
        .map(|row| Actor::try_from(row).map_err(|e| AppError::Internal(e.to_string())))
        .collect()
}

pub async fn search_users(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<User>>> {
    let limit = query.limit.unwrap_or(20).min(100);

    let rows = sqlx::query_as::<_, ActorRow>(&format!(
        "SELECT {ACTOR_COLUMNS} FROM users
         WHERE username_lower ILIKE '%' || lower($1) || '%'
            OR COALESCE(name, '') ILIKE '%' || $1 || '%'
         LIMIT $2"
    ))
    .bind(&query.q)
    .bind(limit as i64)
    .fetch_all(state.db())
    .await
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
    let rows = if let Some(ref me) = exclude {
        sqlx::query_as::<_, ActorRow>(&format!(
            "SELECT {ACTOR_COLUMNS} FROM users
             WHERE id <> $1
             ORDER BY followers_count DESC, notes_count DESC
             LIMIT $2"
        ))
        .bind(me)
        .bind((limit * 2) as i64)
        .fetch_all(state.db())
        .await
    } else {
        sqlx::query_as::<_, ActorRow>(&format!(
            "SELECT {ACTOR_COLUMNS} FROM users
             ORDER BY followers_count DESC, notes_count DESC
             LIMIT $1"
        ))
        .bind((limit * 2) as i64)
        .fetch_all(state.db())
        .await
    }
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
    let rows = sqlx::query_as::<_, ActorRow>(
        "SELECT u.* FROM users u
         JOIN follows f ON u.id = f.follower_id
         WHERE f.followee_id = $1 AND NOT f.is_accepted",
    )
    .bind(auth.user_id.to_string())
    .fetch_all(state.db())
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let users: Vec<User> = actors_from_rows(rows)?.iter().map(actor_to_user).collect();
    Ok(Json(users))
}

pub async fn accept_follow_request(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<UserRelation>> {
    let target_id = parse_actor_id(&id)?;
    let me = auth.user_id.to_string();
    let target = target_id.to_string();

    let mut tx = state
        .db()
        .begin()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    sqlx::query(
        "UPDATE follows SET is_accepted = true
         WHERE follower_id = $1 AND followee_id = $2",
    )
    .bind(&target)
    .bind(&me)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    sqlx::query("UPDATE users SET following_count = following_count + 1 WHERE id = $1")
        .bind(&target)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    sqlx::query("UPDATE users SET followers_count = followers_count + 1 WHERE id = $1")
        .bind(&me)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let notif = Notification::new(
        crate::models::notification::NotificationType::FollowRequestAccepted,
        target_id,
        Some(auth.user_id),
        None,
    );
    publish_notification(&state, &notif, None, None).await;

    Ok(Json(build_relation(&state, auth.user_id, target_id).await?))
}

pub async fn reject_follow_request(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let target_id = parse_actor_id(&id)?;
    sqlx::query(
        "DELETE FROM follows
         WHERE follower_id = $1 AND followee_id = $2 AND NOT is_accepted",
    )
    .bind(target_id.to_string())
    .bind(auth.user_id.to_string())
    .execute(state.db())
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
    unfollow(state.db(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}
