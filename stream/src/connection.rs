use crate::{ChannelRegistry, ClientMessage, StreamMessage};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// Represents a single WebSocket client connection to the stream.
pub struct StreamConnection {
    pub user_id: Option<String>,
    surreal: mithic_db::SurrealClient,
    dragonfly: mithic_db::DragonflyClient,
    channels: ChannelRegistry,
    tx: UnboundedSender<StreamMessage>,
}

impl StreamConnection {
    pub fn new(
        user_id: Option<String>,
        surreal: mithic_db::SurrealClient,
        dragonfly: mithic_db::DragonflyClient,
    ) -> (Self, UnboundedReceiver<StreamMessage>) {
        let (tx, rx) = unbounded_channel();
        let conn = Self {
            user_id,
            surreal,
            dragonfly,
            channels: ChannelRegistry::new(),
            tx,
        };
        (conn, rx)
    }

    pub async fn handle_message(&self, msg: ClientMessage) {
        match msg {
            ClientMessage::Connect {
                id,
                channel,
                params,
                ..
            } => {
                if let Some(ch) = self
                    .channels
                    .create_channel(&channel, id.clone(), self.user_id.clone(), params)
                    .await
                {
                    let (ch_tx, mut ch_rx) = unbounded_channel();
                    ch.set_sender(ch_tx);

                    let surreal = self.surreal.clone();
                    let dragonfly = self.dragonfly.clone();
                    let connection_tx = self.tx.clone();
                    let ch_id = id.clone();

                    if let Err(e) = ch.init(&surreal, &dragonfly).await {
                        tracing::error!("Failed to initialize stream channel {}: {}", channel, e);
                    } else {
                        tokio::spawn(async move {
                            while let Some(msg) = ch_rx.recv().await {
                                let ev = StreamMessage::ChannelEvent {
                                    id: ch_id.clone(),
                                    body: msg,
                                };
                                if connection_tx.send(ev).is_err() {
                                    break;
                                }
                            }
                        });
                        let _ = self.tx.send(StreamMessage::Connected { id });
                    }
                }
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
