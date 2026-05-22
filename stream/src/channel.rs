//! Channel trait for streaming
//!
//! Defines the interface for all stream channels.

use async_trait::async_trait;
use mithic_db::DragonflyClient;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

/// Channel message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub body: serde_json::Value,
}

impl ChannelMessage {
    pub fn new(msg_type: impl Into<String>, body: impl Serialize) -> Self {
        Self {
            msg_type: msg_type.into(),
            body: serde_json::to_value(body).unwrap_or_default(),
        }
    }
}

/// Channel trait
#[async_trait]
pub trait Channel: Send + Sync {
    /// Channel name
    fn name(&self) -> &str;

    /// Channel ID
    fn id(&self) -> &str;

    /// Initialize the channel
    async fn init(&self, redis: &DragonflyClient) -> anyhow::Result<()>;

    /// Handle incoming message from client
    async fn on_message(&self, msg_type: &str, body: serde_json::Value);

    /// Dispose/ cleanup
    async fn dispose(&self);

    /// Send message to client
    fn sender(&self) -> &UnboundedSender<ChannelMessage>;

    /// Set sender
    fn set_sender(&self, sender: UnboundedSender<ChannelMessage>);

    /// Check if channel requires authentication
    fn require_credential(&self) -> bool {
        false
    }

    /// Check if channel can be shared
    fn should_share(&self) -> bool {
        false
    }
}

/// Channel base struct
pub struct ChannelBase {
    pub id: String,
    pub name: String,
    pub sender: Option<UnboundedSender<ChannelMessage>>,
}

impl ChannelBase {
    pub fn new(id: String, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            sender: None,
        }
    }

    pub fn send(&self, msg: ChannelMessage) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(msg);
        }
    }
}
