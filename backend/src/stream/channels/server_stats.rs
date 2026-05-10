//! Server statistics streaming channel
//!
//! Provides real-time server statistics for monitoring.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    db::DragonflyClient,
    stream::{
        channel::{Channel, ChannelBase, ChannelMessage},
        channels::SharedState,
    },
};

/// Server stats channel for monitoring server status
pub struct ServerStatsChannel {
    base: SharedState,
}

impl ServerStatsChannel {
    pub fn new(id: String) -> Self {
        Self {
            base: Arc::new(Mutex::new(ChannelBase::new(id, "serverStats"))),
        }
    }
}

#[async_trait]
impl Channel for ServerStatsChannel {
    fn name(&self) -> &str {
        "serverStats"
    }

    fn id(&self) -> &str {
        let base = self.base.blocking_lock();
        &base.id
    }

    async fn init(&self, _redis: &DragonflyClient) -> anyhow::Result<()> {
        // Subscribe to server statistics
        Ok(())
    }

    async fn on_message(&self, msg_type: &str, body: serde_json::Value) {
        let base = self.base.lock().await;
        let msg = ChannelMessage::new(msg_type, body);
        base.send(msg);
    }

    async fn dispose(&self) {
        // Cleanup
    }

    fn sender(&self) -> &tokio::sync::mpsc::UnboundedSender<ChannelMessage> {
        let base = self.base.blocking_lock();
        base.sender.as_ref().expect("Sender not set")
    }

    fn set_sender(&self, sender: tokio::sync::mpsc::UnboundedSender<ChannelMessage>) {
        let mut base = self.base.blocking_lock();
        base.sender = Some(sender);
    }

    fn require_credential(&self) -> bool {
        true // Server stats require authentication
    }
}
