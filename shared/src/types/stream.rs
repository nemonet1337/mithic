use serde::{Deserialize, Serialize};

use super::{Note, Notification};

/// `/api/streaming` WebSocket で配信されるイベント。
/// フロントエンドの `StreamEvent` と同一のワイヤフォーマット。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StreamEvent {
    Note { note: Box<Note> },
    Notification { notification: Box<Notification> },
}
