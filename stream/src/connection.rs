use crate::{ChannelRegistry, ClientMessage, StreamMessage};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// Represents a single WebSocket client connection to the stream.
pub struct StreamConnection {
    pub user_id: Option<String>,
    channels: ChannelRegistry,
    tx: UnboundedSender<StreamMessage>,
}

impl StreamConnection {
    pub fn new(user_id: Option<String>) -> (Self, UnboundedReceiver<StreamMessage>) {
        let (tx, rx) = unbounded_channel();
        let conn = Self {
            user_id,
            channels: ChannelRegistry::new(),
            tx,
        };
        (conn, rx)
    }

    pub async fn handle_message(&self, msg: ClientMessage) {
        match msg {
            ClientMessage::Connect { id, .. } => {
                let _ = self.tx.send(StreamMessage::Connected { id });
            }
            ClientMessage::Disconnect { id } => {
                self.channels.remove_channel(&id).await;
            }
            ClientMessage::ChannelMsg { id, msg_type, body } => {
                if let Some(ch) = self.channels.get_channel(&id).await {
                    ch.on_message(&msg_type, body).await;
                }
            }
            _ => {}
        }
    }

    pub fn send(&self, msg: StreamMessage) {
        let _ = self.tx.send(msg);
    }

    pub fn channels(&self) -> &ChannelRegistry {
        &self.channels
    }
}
