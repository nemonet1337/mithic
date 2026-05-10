//! Queue statistics streaming channel
//!
//! Provides real-time queue statistics for monitoring.

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

/// Queue stats channel for monitoring queue status
pub struct QueueStatsChannel {
    base: SharedState,
}

impl QueueStatsChannel {
    pub fn new(id: String) -> Self {
        Self {
            base: Arc::new(Mutex::new(ChannelBase::new(id, "queueStats"))),
        }
    }
}

#[async_trait]
impl Channel for QueueStatsChannel {
    fn name(&self) -> &str {
        "queueStats"
    }

    fn id(&self) -> &str {
        let base = self.base.blocking_lock();
        &base.id
    }

    async fn init(&self, _redis: &DragonflyClient) -> anyhow::Result<()> {
        // Subscribe to queue statistics
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
        true // Queue stats require authentication
    }
}
