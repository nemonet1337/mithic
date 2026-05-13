//! User List Channel
//!
//! Real-time user list update notifications.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use tracing::info;

use crate::{
    db::DragonflyClient,
    stream::channel::{Channel, ChannelBase, ChannelMessage},
};

pub struct UserListChannel {
    base: Arc<Mutex<ChannelBase>>,
    list_id: String,
    user_id: Option<String>,
    redis_subscription: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl UserListChannel {
    pub fn new(id: String, list_id: String, user_id: Option<String>) -> Self {
        Self {
            base: Arc::new(Mutex::new(ChannelBase::new(id, "userList"))),
            list_id,
            user_id,
            redis_subscription: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Channel for UserListChannel {
    fn name(&self) -> &str {
        "userList"
    }

    fn id(&self) -> &str {
        let base = self.base.blocking_lock();
        &base.id
    }

    async fn init(&self, redis: &DragonflyClient) -> anyhow::Result<()> {
        let list_id = self.list_id.clone();
        let base = self.base.clone();
        let mut redis_client = redis.client.clone();

        let handle = tokio::spawn(async move {
            let mut pubsub = match redis_client.get_async_pubsub().await {
                Ok(p) => p,
                Err(_) => return,
            };

            let channel = format!("user-list:{}", list_id);
            let _ = pubsub.subscribe(&channel).await;

            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                let payload: String = msg.get_payload().unwrap_or_default();
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&payload) {
                    let event_type = data.get("type").and_then(|v| v.as_str()).unwrap_or("note");
                    let base = base.lock().await;
                    base.send(ChannelMessage::new(event_type, data));
                }
            }
        });

        let mut sub = self.redis_subscription.lock().await;
        *sub = Some(handle);

        info!("User list channel initialized for list {}", self.list_id);
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
        false
    }
}
