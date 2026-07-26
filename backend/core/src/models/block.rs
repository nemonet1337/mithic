use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

pub type BlockId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: BlockId,
    pub blocker_id: ActorId,
    pub blockee_id: ActorId,
    pub created_at: DateTime<Utc>,
}

impl Block {
    pub fn new(blocker_id: ActorId, blockee_id: ActorId) -> Self {
        Self {
            id: BlockId::new(),
            blocker_id,
            blockee_id,
            created_at: Utc::now(),
        }
    }
}

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
    fn from(b: Block) -> Self {
        Self {
            id: b.id.to_string(),
            blocker_id: b.blocker_id.to_string(),
            blockee_id: b.blockee_id.to_string(),
            created_at: b.created_at,
        }
    }
}

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
