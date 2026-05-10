use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{actor::ActorId, note::NoteId};

/// リアクションID
pub type ReactionId = Ulid;

/// リアクションモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    pub id: ReactionId,

    pub created_at: DateTime<Utc>,

    /// 対象ノートID
    pub note_id: NoteId,

    /// リアクションしたユーザーID
    pub actor_id: ActorId,

    /// リアクション種別（絵文字名、または"like"）
    pub reaction: String,

    /// リモートからのリアクションか
    pub is_remote: bool,

    /// ActivityPub URI（リモート用）
    pub uri: Option<String>,
}

impl Reaction {
    /// 新しいリアクションを作成
    pub fn new(note_id: NoteId, actor_id: ActorId, reaction: String) -> Self {
        Self {
            id: ReactionId::new(),
            created_at: Utc::now(),
            note_id,
            actor_id,
            reaction,
            is_remote: false,
            uri: None,
        }
    }

    /// リモートリアクションを作成
    pub fn new_remote(
        note_id: NoteId,
        actor_id: ActorId,
        reaction: String,
        uri: String,
    ) -> Self {
        Self {
            id: ReactionId::new(),
            created_at: Utc::now(),
            note_id,
            actor_id,
            reaction,
            is_remote: true,
            uri: Some(uri),
        }
    }
}
