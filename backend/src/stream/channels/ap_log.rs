//! ActivityPub Log Channel
//!
//! Real-time ActivityPub event log stream for admins.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use tracing::info;

use crate::{
    db::DragonflyClient,
    stream::channel::{Channel, ChannelBase, ChannelMessage},
};

pub struct ApLogChannel {
    base: Arc<Mutex<ChannelBase>>,
    redis_subscription: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl ApLogChannel {
    pub fn new(id: String) -> Self {
        Self {
            base: Arc::new(Mutex::new(ChannelBase::new(id, "apLog"))),
            redis_subscription: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Channel for ApLogChannel {
    fn name(&self) -> &str {
        "apLog"
    }

    fn id(&self) -> &str {
        let base = self.base.blocking_lock();
        &base.id
    }

    async fn init(&self, redis: &DragonflyClient) -> anyhow::Result<()> {
        let base = self.base.clone();
        let mut redis_client = redis.client.clone();

        let handle = tokio::spawn(async move {
            let mut pubsub = match redis_client.get_async_pubsub().await {
                Ok(p) => p,
                Err(_) => return,
            };

            let _ = pubsub.subscribe("ap:log").await;

            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                let payload: String = msg.get_payload().unwrap_or_default();
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&payload) {
                    let base = base.lock().await;
                    base.send(ChannelMessage::new("apLog", data));
                }
            }
        });

        let mut sub = self.redis_subscription.lock().await;
        *sub = Some(handle);

        info!("ActivityPub log channel initialized");
        Ok(())
    }

    async fn on_message(&self, _msg_type: &str, _body: serde_json::Value) {}

    async fn dispose(&self) {
        let mut sub = self.redis_subscription.lock().await;
        if let Some(handle) = sub.take() {
            handle.abort();
        }
    }

    fn sender(&self) -> &UnboundedSender<ChannelMessage> {
        panic!("Use base.send() instead");
    }

    fn set_sender(&self, _sender: UnboundedSender<ChannelMessage>) {}

    fn require_credential(&self) -> bool {
        true
    }

    fn should_share(&self) -> bool {
        true
    }
}
