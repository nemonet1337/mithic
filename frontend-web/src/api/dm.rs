use serde::{Deserialize, Serialize};

use super::client::{ApiError, request};
use crate::models::User;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectMessage {
    pub id: String,
    pub content: String,
    pub sender_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub participant: User,
    pub last_message: Option<DirectMessage>,
    pub unread_count: u32,
}

pub async fn fetch_conversations(token: &str) -> Result<Vec<Conversation>, ApiError> {
    request::<Vec<Conversation>, ()>("GET", "conversations", Some(token), None).await
}

pub async fn fetch_messages(
    token: &str,
    conversation_id: &str,
    before_id: Option<&str>,
) -> Result<Vec<DirectMessage>, ApiError> {
    let path = match before_id {
        Some(id) => format!(
            "conversations/{}/messages?limit=30&cursor={}",
            conversation_id, id
        ),
        None => format!("conversations/{}/messages?limit=30", conversation_id),
    };
    request::<Vec<DirectMessage>, ()>("GET", &path, Some(token), None).await
}

pub async fn send_message(
    token: &str,
    conversation_id: &str,
    content: &str,
) -> Result<DirectMessage, ApiError> {
    #[derive(Serialize)]
    struct Body<'a> {
        text: &'a str,
    }
    request(
        "POST",
        &format!("conversations/{}/messages", conversation_id),
        Some(token),
        Some(&Body { text: content }),
    )
    .await
}

pub async fn create_conversation(token: &str, user_id: &str) -> Result<Conversation, ApiError> {
    #[derive(Serialize)]
    struct Body<'a> {
        participant_ids: Vec<&'a str>,
    }
    request(
        "POST",
        "conversations",
        Some(token),
        Some(&Body { participant_ids: vec![user_id] }),
    )
    .await
}
