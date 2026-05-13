use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{actor::ActorId, note::NoteId};

pub type UserNotePiningId = Ulid;

/// ユーザーのピン留めノート管理
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserNotePining {
    pub id: UserNotePiningId,

    /// ピン留めしたユーザーのID
    pub user_id: ActorId,

    /// ピン留めされたノートのID
    pub note_id: NoteId,

    pub created_at: DateTime<Utc>,
}

impl UserNotePining {
    pub fn new(user_id: ActorId, note_id: NoteId) -> Self {
        Self {
            id: UserNotePiningId::new(),
            user_id,
            note_id,
            created_at: Utc::now(),
        }
    }
}
