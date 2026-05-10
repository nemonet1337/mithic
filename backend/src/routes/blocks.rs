//! Block API endpoints
//!
//! Provides API for blocking and unblocking users.

use axum::{
    extract::{Query, State},
    Json,
};
use tracing::{error, info, warn};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{Actor, Block, BlockResponse, CreateBlockRequest, BlockListQuery},
    state::{AppState, AuthUser},
};

/// Create a new block
pub async fn create_block(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateBlockRequest>,
) -> Result<Json<BlockResponse>> {
    let blockee_id = Ulid::from_string(&req.user_id)
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;

    // Check if user exists
    let blockee: Option<Actor> = state
        .surreal()
        .select(("user", blockee_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    if blockee.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    let blockee = blockee.unwrap();

    // Can't block yourself
    if auth_user.user_id == blockee.id {
        return Err(AppError::Validation("Cannot block yourself".to_string()));
    }

    // Check if already blocked
    let existing: Option<Block> = state
        .surreal()
        .query("SELECT * FROM block WHERE blocker_id = $blocker_id AND blockee_id = $blockee_id")
        .bind(("blocker_id", auth_user.user_id.to_string()))
        .bind(("blockee_id", blockee.id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .ok()
        .flatten();

    if existing.is_some() {
        return Err(AppError::Validation("Already blocked".to_string()));
    }

    // Remove any existing follow relationships in both directions
    let _ = state
        .surreal()
        .query("DELETE follow WHERE (in = $user1 AND out = $user2) OR (in = $user2 AND out = $user1)")
        .bind(("user1", auth_user.user_id.to_string()))
        .bind(("user2", blockee.id.to_string()))
        .await;

    // Create block relationship using RELATE
    let block = Block::new(auth_user.user_id, blockee.id.clone());
    
    state.surreal()
        .query("RELATE user:$blocker->block->user:$blockee SET created_at = $created_at")
        .bind(("blocker", auth_user.user_id.to_string()))
        .bind(("blockee", blockee.id.to_string()))
        .bind(("created_at", block.created_at))
        .await
        .map_err(|e| {
            error!("Failed to create block: {}", e);
            AppError::Database(e)
        })?;

    // If blockee is remote, send Reject Follow activity if they were following us
    if blockee.host.is_some() {
        // TODO: Send Reject Follow activity via federation
        warn!("Remote user block - Reject Follow not yet implemented");
    }

    info!(
        "User {} blocked user {}",
        auth_user.user_id, blockee.id
    );

    Ok(Json(BlockResponse::from(block)))
}

/// Delete a block (unblock)
pub async fn delete_block(
    auth_user: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(block_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>> {
    let block_ulid = Ulid::from_string(&block_id)
        .map_err(|_| AppError::Validation("Invalid block ID".to_string()))?;

    // Find the block relationship
    let block: Option<Block> = state
        .surreal()
        .select(("block", block_ulid.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let block = block.ok_or_else(|| AppError::NotFound("Block not found".to_string()))?;

    // Verify ownership
    if block.blocker_id != auth_user.user_id {
        return Err(AppError::Forbidden("Not authorized".to_string()));
    }

    // Delete the block relationship
    state.surreal()
        .query("DELETE block WHERE in = $blocker_id AND out = $blockee_id")
        .bind(("blocker_id", block.blocker_id.to_string()))
        .bind(("blockee_id", block.blockee_id.to_string()))
        .await
        .map_err(|e| {
            error!("Failed to delete block: {}", e);
            AppError::Database(e)
        })?;

    info!(
        "User {} unblocked user {}",
        auth_user.user_id, block.blockee_id
    );

    Ok(Json(serde_json::json!({"success": true})))
}

/// Get list of blocked users
pub async fn get_blocks(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<BlockListQuery>,
) -> Result<Json<Vec<BlockResponse>>> {
    let limit = query.limit();
    let offset = query.offset();

    let blocks: Vec<Block> = state
        .surreal()
        .query("SELECT * FROM block WHERE blocker_id = $blocker_id LIMIT $limit START $offset")
        .bind(("blocker_id", auth_user.user_id.to_string()))
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    let responses: Vec<BlockResponse> = blocks
        .into_iter()
        .map(BlockResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Check if a user is blocked
pub async fn is_blocking(
    auth_user: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>> {
    let target_id = Ulid::from_string(&user_id)
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;

    let block: Option<Block> = state
        .surreal()
        .query("SELECT * FROM block WHERE blocker_id = $blocker_id AND blockee_id = $blockee_id")
        .bind(("blocker_id", auth_user.user_id.to_string()))
        .bind(("blockee_id", target_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .ok()
        .flatten();

    Ok(Json(serde_json::json!({
        "blocking": block.is_some()
    })))
}
