//! Admin: accounts suspend / relays

use crate::db::queries::{
    create_relay, delete_relay, get_relay_by_id, get_relay_by_inbox, list_relays,
    update_relay_status,
};
use crate::models::actor::ActorId;
use crate::{AppError, AuthUser, Result};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use shared::Relay;

use crate::routes::v1::common::parse_actor_id;
use crate::state::AppState;

fn require_admin(auth: &AuthUser) -> Result<()> {
    if auth.is_admin {
        Ok(())
    } else {
        Err(AppError::Forbidden("Admin only".to_string()))
    }
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

pub async fn suspend(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    require_admin(&auth)?;
    let target_id = parse_actor_id(&id)?;
    sqlx::query("UPDATE users SET is_suspended = true WHERE id = $1")
        .bind(target_id.to_string())
        .execute(state.db())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unsuspend(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    require_admin(&auth)?;
    let target_id = parse_actor_id(&id)?;
    sqlx::query("UPDATE users SET is_suspended = false WHERE id = $1")
        .bind(target_id.to_string())
        .execute(state.db())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_account(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    require_admin(&auth)?;
    let target_id: ActorId = parse_actor_id(&id)?;
    let id = target_id.to_string();

    // 子テーブルは FK ON DELETE CASCADE で消えるが、明示 DELETE を順に実行して
    // CASCADE 対象外のテーブルも確実に片付ける。
    let mut tx = state
        .db()
        .begin()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let deletes: [&str; 10] = [
        "DELETE FROM note_reactions WHERE actor_id = $1",
        "DELETE FROM poll_votes WHERE actor_id = $1",
        "DELETE FROM bookmarks WHERE user_id = $1",
        "DELETE FROM user_note_pins WHERE user_id = $1",
        "DELETE FROM notifications WHERE user_id = $1 OR notifier_id = $1",
        "DELETE FROM drive_files WHERE owner_id = $1",
        "DELETE FROM notes WHERE actor_id = $1",
        "DELETE FROM follows WHERE follower_id = $1 OR followee_id = $1",
        "DELETE FROM blocks WHERE blocker_id = $1 OR blocked_id = $1",
        "DELETE FROM mutes WHERE muter_id = $1 OR muted_id = $1",
    ];
    for sql in deletes {
        sqlx::query(sql)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Relays
// ---------------------------------------------------------------------------

fn to_dto(r: crate::models::relay::Relay) -> Relay {
    Relay {
        id: r.id.to_string(),
        inbox: r.inbox,
        status: r.status,
        created_at: r.created_at.to_rfc3339(),
        updated_at: r.updated_at.map(|t| t.to_rfc3339()),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRelayRequest {
    pub inbox: String,
}

pub async fn list_relays_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<Relay>>> {
    require_admin(&auth)?;
    let relays = list_relays(state.db(), 100)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(relays.into_iter().map(to_dto).collect()))
}

pub async fn add_relay(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<AddRelayRequest>,
) -> Result<Json<Relay>> {
    require_admin(&auth)?;
    if let Ok(Some(_)) = get_relay_by_inbox(state.db(), &request.inbox).await {
        return Err(AppError::Validation("Relay already exists".to_string()));
    }
    create_relay(state.db(), &request.inbox)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let relay = crate::models::relay::Relay::new(request.inbox);
    Ok(Json(to_dto(relay)))
}

pub async fn remove_relay(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    require_admin(&auth)?;
    delete_relay(state.db(), &id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRelayRequest {
    pub status: Option<shared::types::relay::RelayStatus>,
}

pub async fn update_relay(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(request): Json<UpdateRelayRequest>,
) -> Result<Json<Relay>> {
    require_admin(&auth)?;
    let mut relay = get_relay_by_id(state.db(), &id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Relay not found".to_string()))?;

    if let Some(status) = request.status {
        relay.status = status;
        update_relay_status(state.db(), &id, relay.status)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        relay.updated_at = Some(chrono::Utc::now());
    }
    Ok(Json(to_dto(relay)))
}
