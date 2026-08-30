use serde::{Deserialize, Serialize};

use super::{Note, Notification};

/// `/api/v1/streaming` WebSocket の正規ワイヤフォーマット。
///
/// ```json
/// { "type": "note", "body": { ...Note } }
/// { "type": "notification", "body": { ...Notification } }
/// { "type": "noteDeleted", "body": { "id": "..." } }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "body", rename_all = "camelCase")]
pub enum StreamEvent {
    Note(Box<Note>),
    Notification(Box<Notification>),
    NoteDeleted { id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_deleted_wire_format() {
        let event = StreamEvent::NoteDeleted {
            id: "01TESTNOTEDELETED000000000".into(),
        };
        let json = serde_json::to_value(&event).expect("serialize");
        assert_eq!(json["type"], "noteDeleted");
        assert_eq!(json["body"]["id"], "01TESTNOTEDELETED000000000");
        let back: StreamEvent = serde_json::from_value(json).expect("deserialize");
        match back {
            StreamEvent::NoteDeleted { id } => assert!(id.starts_with("01")),
            _ => panic!("wrong variant"),
        }
    }
}
