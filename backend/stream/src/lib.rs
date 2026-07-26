//! WebSocket Streaming Module
//!
//! Provides real-time streaming of notifications, timeline updates, and more.
//! Based on Misskey's streaming API architecture.

pub mod channel;
pub mod channels;
pub mod connection;

pub use channel::{Channel, ChannelMessage};
pub use channels::{
    AdminChannel, ApLogChannel, DriveChannel, GlobalTimelineChannel, HashtagChannel,
    HomeTimelineChannel, QueueStatsChannel, ServerStatsChannel, UserListChannel,
};
pub use connection::StreamConnection;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Stream message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StreamMessage {
    /// Channel event
    #[serde(rename = "channel")]
    ChannelEvent {
        id: String,
        #[serde(flatten)]
        body: ChannelMessage,
    },
    /// Connected confirmation
    #[serde(rename = "connected")]
    Connected { id: String },
    /// Note updated
    #[serde(rename = "noteUpdated")]
    NoteUpdated {
        id: String,
        update_type: String,
        body: serde_json::Value,
    },
    /// Error
    #[serde(rename = "error")]
    Error { message: String },
}

/// Client to server messages
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    /// Connect to channel
    /// `id` 省略時は channel 名を id として使う (WebUI 簡略接続向け)
    #[serde(rename = "connect")]
    Connect {
        #[serde(default)]
        id: String,
        channel: String,
        params: Option<serde_json::Value>,
        pong: Option<bool>,
    },
    /// Disconnect from channel
    #[serde(rename = "disconnect")]
    Disconnect { id: String },
    /// Send message to channel
    #[serde(rename = "channel")]
    ChannelMsg {
        id: String,
        msg_type: String,
        body: serde_json::Value,
    },
    /// Subscribe to note updates
    #[serde(rename = "subNote")]
    SubNote { id: String, read: Option<bool> },
    /// Unsubscribe from note updates
    #[serde(rename = "unsubNote")]
    UnsubNote { id: String },
    /// Mark notification as read
    #[serde(rename = "readNotification")]
    ReadNotification { id: String },
}

/// Channel registry
#[derive(Clone)]
pub struct ChannelRegistry {
    channels: Arc<RwLock<HashMap<String, Arc<dyn Channel + Send + Sync>>>>,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_channel(
        &self,
        channel_type: &str,
        id: String,
        user_id: Option<String>,
        params: Option<serde_json::Value>,
    ) -> Option<Arc<dyn Channel + Send + Sync>> {
        let channel: Arc<dyn Channel + Send + Sync> = match channel_type {
            "homeTimeline" => Arc::new(HomeTimelineChannel::new(id.clone(), user_id.clone())),
            "globalTimeline" => Arc::new(GlobalTimelineChannel::new(id.clone())),
            "hashtag" => {
                let tag = params
                    .as_ref()
                    .and_then(|p| p.get("q").and_then(|q| q.as_str()))
                    .unwrap_or("")
                    .to_string();
                Arc::new(HashtagChannel::new(id.clone(), tag))
            }
            "admin" => Arc::new(AdminChannel::new(id.clone())),
            "queueStats" => Arc::new(QueueStatsChannel::new(id.clone())),
            "serverStats" => Arc::new(ServerStatsChannel::new(id.clone())),
            "drive" => Arc::new(DriveChannel::new(id.clone(), user_id.clone())),
            "apLog" => Arc::new(ApLogChannel::new(id.clone())),
            "userList" => {
                let list_id = params
                    .as_ref()
                    .and_then(|p| p.get("listId").and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .to_string();
                Arc::new(UserListChannel::new(id.clone(), list_id, user_id.clone()))
            }
            _ => return None,
        };

        let mut channels = self.channels.write().await;
        channels.insert(id, channel.clone());
        Some(channel)
    }

    pub async fn remove_channel(&self, id: &str) {
        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.remove(id) {
            channel.dispose().await;
        }
    }

    pub async fn get_channel(&self, id: &str) -> Option<Arc<dyn Channel + Send + Sync>> {
        let channels = self.channels.read().await;
        channels.get(id).cloned()
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MessagePack serialization support (P-G15)
// ---------------------------------------------------------------------------

/// Serialize a stream message using MessagePack if accepted by the client
pub fn serialize_msgpack<T: Serialize>(msg: &T) -> anyhow::Result<Vec<u8>> {
    rmp_serde::to_vec(msg).map_err(|e| anyhow::anyhow!("MessagePack encode error: {}", e))
}
