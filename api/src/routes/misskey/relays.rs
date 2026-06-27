use crate::state::AppState;
use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{
    create_relay, delete_relay, get_relay_by_id, get_relay_by_inbox, list_relays, update_relay_status,
};
use serde::Deserialize;
use shared::Relay;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRelayRequest {
    pub inbox: String,
}

pub async fn add_relay(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Json(request): Json<AddRelayRequest>,
) -> Result<Json<Relay>> {
    if let Ok(Some(_)) = get_relay_by_inbox(state.surreal(), &request.inbox).await {
        return Err(AppError::Validation("Relay already exists".to_string()));
    }
    
    create_relay(state.surreal(), &request.inbox)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let relay = mithic_core::models::relay::Relay::new(request.inbox);
    Ok(Json(Relay {
        id: relay.id.to_string(),
        inbox: relay.inbox,
        status: status_to_dto(relay.status),
        created_at: relay.created_at.to_rfc3339(),
        updated_at: relay.updated_at.map(|t| t.to_rfc3339()),
    }))
}

fn status_to_dto(status: mithic_core::models::relay::RelayStatus) -> shared::types::relay::RelayStatus {
    match status {
        mithic_core::models::relay::RelayStatus::Requesting => shared::types::relay::RelayStatus::Requesting,
        mithic_core::models::relay::RelayStatus::Accepted => shared::types::relay::RelayStatus::Accepted,
        mithic_core::models::relay::RelayStatus::Rejected => shared::types::relay::RelayStatus::Rejected,
    }
}

fn status_from_dto(status: shared::types::relay::RelayStatus) -> mithic_core::models::relay::RelayStatus {
    match status {
        shared::types::relay::RelayStatus::Requesting => mithic_core::models::relay::RelayStatus::Requesting,
        shared::types::relay::RelayStatus::Accepted => mithic_core::models::relay::RelayStatus::Accepted,
        shared::types::relay::RelayStatus::Rejected => mithic_core::models::relay::RelayStatus::Rejected,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayIdRequest {
    pub relay_id: String,
}

pub async fn list_relays_route(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
) -> Result<Json<Vec<Relay>>> {
    let relays = list_relays(state.surreal(), 100)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(relays.into_iter().map(|r| Relay {
        id: r.id.to_string(),
        inbox: r.inbox,
        status: status_to_dto(r.status),
        created_at: r.created_at.to_rfc3339(),
        updated_at: r.updated_at.map(|t| t.to_rfc3339()),
    }).collect()))
}

pub async fn remove_relay(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Json(request): Json<RelayIdRequest>,
) -> Result<StatusCode> {
    delete_relay(state.surreal(), &request.relay_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRelayRequest {
    pub relay_id: String,
    pub status: Option<shared::types::relay::RelayStatus>,
}

pub async fn update_relay(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Json(request): Json<UpdateRelayRequest>,
) -> Result<Json<Relay>> {
    let relay = get_relay_by_id(state.surreal(), &request.relay_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Relay not found".to_string()))?;

    let status = request.status.map(status_from_dto).unwrap_or(relay.status);
    update_relay_status(state.surreal(), &request.relay_id, status)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(Relay {
        id: relay.id.to_string(),
        inbox: relay.inbox,
        status: status_to_dto(status),
        created_at: relay.created_at.to_rfc3339(),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    }))
}