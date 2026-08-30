//! Admin: accounts suspend / relays

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use mithic_core::models::actor::ActorId;
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{
    create_relay, delete_relay, get_relay_by_id, get_relay_by_inbox, list_relays,
    update_relay_status,
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
    state
        .surreal()
        .query("UPDATE user SET is_suspended = true WHERE id = type::record('user', $id)")
        .bind(("id", target_id.to_string()))
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
    state
        .surreal()
        .query("UPDATE user SET is_suspended = false WHERE id = type::record('user', $id)")
        .bind(("id", target_id.to_string()))
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

    state
        .surreal()
        .query(
            r#"
            LET $u = type::record('user', $id);

            DELETE note_reaction WHERE actor_id = $u;
            DELETE poll_vote WHERE actor_id = $u;
            DELETE bookmark WHERE user_id = $u;
            DELETE user_note_pining WHERE in = $u;
            DELETE notification WHERE user_id = $u OR notifier_id = $u;
            DELETE drive_file WHERE user_id = $u;
            DELETE note WHERE actor_id = $u;
            DELETE follow WHERE in = $u OR out = $u;
            DELETE block WHERE in = $u OR out = $u;
            DELETE mute WHERE in = $u OR out = $u;
            DELETE user WHERE id = $u;
            "#,
        )
        .bind(("id", id))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Relays
// ---------------------------------------------------------------------------

fn status_to_dto(
    status: mithic_core::models::relay::RelayStatus,
) -> shared::types::relay::RelayStatus {
    match status {
        mithic_core::models::relay::RelayStatus::Requesting => {
            shared::types::relay::RelayStatus::Requesting
        }
        mithic_core::models::relay::RelayStatus::Accepted => {
            shared::types::relay::RelayStatus::Accepted
        }
        mithic_core::models::relay::RelayStatus::Rejected => {
            shared::types::relay::RelayStatus::Rejected
        }
    }
}

fn status_from_dto(
    status: shared::types::relay::RelayStatus,
) -> mithic_core::models::relay::RelayStatus {
    match status {
        shared::types::relay::RelayStatus::Requesting => {
            mithic_core::models::relay::RelayStatus::Requesting
        }
        shared::types::relay::RelayStatus::Accepted => {
            mithic_core::models::relay::RelayStatus::Accepted
        }
        shared::types::relay::RelayStatus::Rejected => {
            mithic_core::models::relay::RelayStatus::Rejected
        }
    }
}

fn to_dto(r: mithic_core::models::relay::Relay) -> Relay {
    Relay {
        id: r.id.to_string(),
        inbox: r.inbox,
        status: status_to_dto(r.status),
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
    let relays = list_relays(state.surreal(), 100)
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
    if let Ok(Some(_)) = get_relay_by_inbox(state.surreal(), &request.inbox).await {
        return Err(AppError::Validation("Relay already exists".to_string()));
    }
    create_relay(state.surreal(), &request.inbox)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let relay = mithic_core::models::relay::Relay::new(request.inbox);
    Ok(Json(to_dto(relay)))
}

pub async fn remove_relay(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    require_admin(&auth)?;
    delete_relay(state.surreal(), &id)
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
    let mut relay = get_relay_by_id(state.surreal(), &id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Relay not found".to_string()))?;

    if let Some(status) = request.status {
        relay.status = status_from_dto(status);
        update_relay_status(state.surreal(), &id, relay.status)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        relay.updated_at = Some(chrono::Utc::now());
    }
    Ok(Json(to_dto(relay)))
}
