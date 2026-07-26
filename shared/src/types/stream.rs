use serde::{Deserialize, Serialize};

use super::{Note, Notification};

/// `/api/v1/streaming` WebSocket の正規ワイヤフォーマット。
///
/// ```json
/// { "type": "note", "body": { ...Note } }
/// { "type": "notification", "body": { ...Notification } }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "body", rename_all = "camelCase")]
pub enum StreamEvent {
    Note(Box<Note>),
    Notification(Box<Notification>),
}
