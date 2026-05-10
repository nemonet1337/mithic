use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{actor::ActorId, note::NoteId, NoteVisibility};

/// リノートID
pub type RenoteId = Ulid;

/// リノート（ブースト）モデル
/// 
/// リノートはNoteと同じテーブルに保存されるが、
/// このモデルはリノート関係を管理するための補助モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Renote {
    pub id: RenoteId,

    pub created_at: DateTime<Utc>,

    /// 対象ノートID（リノート元）
    pub note_id: NoteId,

    /// リノートしたユーザーID
    pub actor_id: ActorId,

    /// リノートとして作成されたノートID
    pub renote_note_id: NoteId,

    /// 引用リノートの場合の本文
    pub text: Option<String>,

    /// リモートからのリノートか
    pub is_remote: bool,

    /// ActivityPub URI（リモート用）
    pub uri: Option<String>,
}

impl Renote {
    /// 新しいリノート関係を作成
    pub fn new(note_id: NoteId, actor_id: ActorId, renote_note_id: NoteId) -> Self {
        Self {
            id: RenoteId::new(),
            created_at: Utc::now(),
            note_id,
            actor_id,
            renote_note_id,
            text: None,
            is_remote: false,
            uri: None,
        }
    }

    /// 引用リノートを作成
    pub fn new_with_text(
        note_id: NoteId,
        actor_id: ActorId,
        renote_note_id: NoteId,
        text: String,
    ) -> Self {
        Self {
            id: RenoteId::new(),
            created_at: Utc::now(),
            note_id,
            actor_id,
            renote_note_id,
            text: Some(text),
            is_remote: false,
            uri: None,
        }
    }
}
