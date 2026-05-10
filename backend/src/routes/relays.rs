//! Relay API endpoints
//!
//! Provides API for managing ActivityPub relay servers.

use axum::{
    extract::{Path, State},
    Json,
};
use tracing::{info, warn};

use crate::{
    error::{AppError, Result},
    models::{CreateRelayRequest, RelayResponse, RelayId},
    state::{AppState, AuthUser},
};

/// List all relays
///
/// Returns a list of all configured relay servers.
pub async fn list_relays(
    _auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<RelayResponse>>> {
    let relays = state.relay_service().list_relays().await?;
    
    let responses: Vec<RelayResponse> = relays
        .into_iter()
        .map(|r| r.into())
        .collect();
    
    Ok(Json(responses))
}

/// Add a relay
///
/// Registers a new relay server by its inbox URL.
pub async fn add_relay(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateRelayRequest>,
) -> Result<Json<RelayResponse>> {
    // Only admins can manage relays
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Only admins can manage relays".to_string()));
    }
    
    info!("Admin {} adding relay: {}", auth_user.username, request.inbox);
    
    let relay = state.relay_service().add_relay(request).await?;
    
    Ok(Json(relay.into()))
}

/// Remove a relay
///
/// Removes a relay server by its inbox URL.
pub async fn remove_relay(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // Only admins can manage relays
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Only admins can manage relays".to_string()));
    }
    
    let inbox = request
        .get("inbox")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing 'inbox' field".to_string()))?;
    
    info!("Admin {} removing relay: {}", auth_user.username, inbox);
    
    state.relay_service().remove_relay(inbox).await?;
    
    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// Accept a relay
///
/// Marks a relay as accepted.
pub async fn accept_relay(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RelayResponse>> {
    // Only admins can manage relays
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Only admins can manage relays".to_string()));
    }
    
    let relay_id = id.parse::<RelayId>()
        .map_err(|_| AppError::Validation("Invalid relay ID".to_string()))?;
    
    info!("Admin {} accepting relay: {}", auth_user.username, id);
    
    let relay = state.relay_service().accept_relay(&relay_id).await?;
    
    Ok(Json(relay.into()))
}

/// Reject a relay
///
/// Marks a relay as rejected.
pub async fn reject_relay(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RelayResponse>> {
    // Only admins can manage relays
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Only admins can manage relays".to_string()));
    }
    
    let relay_id = id.parse::<RelayId>()
        .map_err(|_| AppError::Validation("Invalid relay ID".to_string()))?;
    
    info!("Admin {} rejecting relay: {}", auth_user.username, id);
    
    let relay = state.relay_service().reject_relay(&relay_id).await?;
    
    Ok(Json(relay.into()))
}
