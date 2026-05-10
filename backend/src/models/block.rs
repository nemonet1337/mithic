//! Block relationship model
//!
//! Represents a blocking relationship between users.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

/// Block relationship ID
pub type BlockId = Ulid;

/// Block relationship model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: BlockId,

    /// Blocker (user who initiated the block)
    pub blocker_id: ActorId,

    /// Blockee (user being blocked)
    pub blockee_id: ActorId,

    /// When the block was created
    pub created_at: DateTime<Utc>,
}

impl Block {
    /// Create a new block relationship
    pub fn new(blocker_id: ActorId, blockee_id: ActorId) -> Self {
        Self {
            id: BlockId::new(),
            blocker_id,
            blockee_id,
            created_at: Utc::now(),
        }
    }
}

/// Block request/response for API
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBlockRequest {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockResponse {
    pub id: String,
    pub blocker_id: String,
    pub blockee_id: String,
    pub created_at: DateTime<Utc>,
}

impl From<Block> for BlockResponse {
    fn from(block: Block) -> Self {
        Self {
            id: block.id.to_string(),
            blocker_id: block.blocker_id.to_string(),
            blockee_id: block.blockee_id.to_string(),
            created_at: block.created_at,
        }
    }
}

/// Block list query parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockListQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl BlockListQuery {
    pub fn limit(&self) -> i32 {
        self.limit.unwrap_or(20).min(100)
    }

    pub fn offset(&self) -> i32 {
        self.offset.unwrap_or(0)
    }
}
