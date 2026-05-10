//! Stream Connection
//!
//! Manages a single WebSocket connection and its channels.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

use crate::{
    db::DragonflyClient,
    stream::{
        channel::ChannelMessage,
        ClientMessage, StreamMessage,
    },
};

/// Stream connection state
pub struct StreamConnection {
    pub user_id: Option<String>,
    channels: Arc<RwLock<HashMap<String, Box<dyn crate::stream::channel::Channel + Send + Sync>>>>,
    tx: mpsc::UnboundedSender<StreamMessage>,
    dragonfly: DragonflyClient,
}

impl StreamConnection {
    pub fn new(
        user_id: Option<String>,
        tx: mpsc::UnboundedSender<StreamMessage>,
        dragonfly: DragonflyClient,
    ) -> Self {
        Self {
            user_id,
            channels: Arc::new(RwLock::new(HashMap::new())),
            tx,
            dragonfly,
        }
    }

    /// Handle incoming client message
    pub async fn handle_message(&self, msg: ClientMessage) {
        match msg {
            ClientMessage::Connect { id, channel, params, pong } => {
                self.connect_channel(id, channel, params, pong).await;
            }
            ClientMessage::Disconnect { id } => {
                self.disconnect_channel(&id).await;
            }
            ClientMessage::ChannelMsg { id, msg_type, body } => {
                self.channel_message(&id, &msg_type, body).await;
            }
            ClientMessage::SubNote { id: note_id, read } => {
                self.subscribe_note(&note_id, read).await;
            }
            ClientMessage::UnsubNote { id: note_id } => {
                self.unsubscribe_note(&note_id).await;
            }
            ClientMessage::ReadNotification { id } => {
                self.read_notification(&id).await;
            }
        }
    }

    /// Connect to a channel
    async fn connect_channel(
        &self,
        id: String,
        channel_type: String,
        params: Option<serde_json::Value>,
        pong: Option<bool>,
    ) {
        // Check if channel requires authentication
        let requires_auth = match channel_type.as_str() {
            "homeTimeline" => true,
            _ => false,
        };

        if requires_auth && self.user_id.is_none() {
            let _ = self.tx.send(StreamMessage::Error {
                message: "Authentication required".to_string(),
            });
            return;
        }

        // Create channel based on type
        let channel: Box<dyn crate::stream::channel::Channel + Send + Sync> = match channel_type.as_str() {
            "homeTimeline" => {
                Box::new(crate::stream::channels::HomeTimelineChannel::new(
                    id.clone(),
                    self.user_id.clone(),
                ))
            }
            "globalTimeline" => {
                Box::new(crate::stream::channels::GlobalTimelineChannel::new(id.clone()))
            }
            "hashtag" => {
                let tag = params
                    .as_ref()
                    .and_then(|p| p.get("q").and_then(|q| q.as_str()))
                    .unwrap_or("")
                    .to_string();
                Box::new(crate::stream::channels::HashtagChannel::new(id.clone(), tag))
            }
            _ => {
                warn!("Unknown channel type: {}", channel_type);
                return;
            }
        };

        // Initialize channel
        if let Err(e) = channel.init(&self.dragonfly).await {
            error!("Failed to initialize channel: {}", e);
            return;
        }

        // Store channel
        let mut channels = self.channels.write().await;
        channels.insert(id.clone(), channel);

        // Send connected confirmation
        if pong.unwrap_or(false) {
            let _ = self.tx.send(StreamMessage::Connected { id });
        }

        info!(
            "Channel {} connected for user {:?}",
            channel_type, self.user_id
        );
    }

    /// Disconnect from a channel
    async fn disconnect_channel(&self, id: &str) {
        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.remove(id) {
            channel.dispose().await;
            info!("Channel {} disconnected", id);
        }
    }

    /// Send message to channel
    async fn channel_message(&self, id: &str, msg_type: &str, body: serde_json::Value) {
        let channels = self.channels.read().await;
        if let Some(channel) = channels.get(id) {
            channel.on_message(msg_type, body).await;
        }
    }

    /// Subscribe to note updates
    async fn subscribe_note(&self, note_id: &str, _read: Option<bool>) {
        info!("Subscribing to note: {}", note_id);
        // TODO: Implement note subscription
    }

    /// Unsubscribe from note updates
    async fn unsubscribe_note(&self, note_id: &str) {
        info!("Unsubscribing from note: {}", note_id);
        // TODO: Implement note unsubscription
    }

    /// Mark notification as read
    async fn read_notification(&self, notification_id: &str) {
        info!("Marking notification as read: {}", notification_id);
        // TODO: Implement notification read
    }

    /// Dispose connection
    pub async fn dispose(&self) {
        let mut channels = self.channels.write().await;
        for (id, channel) in channels.drain() {
            channel.dispose().await;
            info!("Channel {} disposed", id);
        }
    }
}
