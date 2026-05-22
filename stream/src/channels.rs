use crate::channel::{Channel, ChannelBase, ChannelMessage};
use async_trait::async_trait;
use mithic_db::DragonflyClient;
use std::sync::OnceLock;
use tokio::sync::mpsc::UnboundedSender;

macro_rules! impl_channel {
    ($name:ident, $channel_name:expr) => {
        #[async_trait]
        impl Channel for $name {
            fn name(&self) -> &str {
                $channel_name
            }
            fn id(&self) -> &str {
                &self.base.id
            }
            async fn init(&self, _redis: &DragonflyClient) -> anyhow::Result<()> {
                Ok(())
            }
            async fn on_message(&self, _msg_type: &str, _body: serde_json::Value) {}
            async fn dispose(&self) {}
            fn sender(&self) -> &UnboundedSender<ChannelMessage> {
                self.sender.get().expect("sender not initialized")
            }
            fn set_sender(&self, sender: UnboundedSender<ChannelMessage>) {
                let _ = self.sender.set(sender);
            }
        }
    };
}

pub struct HomeTimelineChannel {
    base: ChannelBase,
    #[allow(dead_code)]
    user_id: Option<String>,
    sender: OnceLock<UnboundedSender<ChannelMessage>>,
}

impl HomeTimelineChannel {
    pub fn new(id: String, user_id: Option<String>) -> Self {
        Self {
            base: ChannelBase::new(id, "homeTimeline"),
            user_id,
            sender: OnceLock::new(),
        }
    }
}

impl_channel!(HomeTimelineChannel, "homeTimeline");

pub struct GlobalTimelineChannel {
    base: ChannelBase,
    sender: OnceLock<UnboundedSender<ChannelMessage>>,
}

impl GlobalTimelineChannel {
    pub fn new(id: String) -> Self {
        Self {
            base: ChannelBase::new(id, "globalTimeline"),
            sender: OnceLock::new(),
        }
    }
}

impl_channel!(GlobalTimelineChannel, "globalTimeline");

pub struct HashtagChannel {
    base: ChannelBase,
    #[allow(dead_code)]
    tag: String,
    sender: OnceLock<UnboundedSender<ChannelMessage>>,
}

impl HashtagChannel {
    pub fn new(id: String, tag: String) -> Self {
        Self {
            base: ChannelBase::new(id, "hashtag"),
            tag,
            sender: OnceLock::new(),
        }
    }
}

impl_channel!(HashtagChannel, "hashtag");

pub struct AdminChannel {
    base: ChannelBase,
    sender: OnceLock<UnboundedSender<ChannelMessage>>,
}

impl AdminChannel {
    pub fn new(id: String) -> Self {
        Self {
            base: ChannelBase::new(id, "admin"),
            sender: OnceLock::new(),
        }
    }
}

impl_channel!(AdminChannel, "admin");

pub struct QueueStatsChannel {
    base: ChannelBase,
    sender: OnceLock<UnboundedSender<ChannelMessage>>,
}

impl QueueStatsChannel {
    pub fn new(id: String) -> Self {
        Self {
            base: ChannelBase::new(id, "queueStats"),
            sender: OnceLock::new(),
        }
    }
}

impl_channel!(QueueStatsChannel, "queueStats");

pub struct ServerStatsChannel {
    base: ChannelBase,
    sender: OnceLock<UnboundedSender<ChannelMessage>>,
}

impl ServerStatsChannel {
    pub fn new(id: String) -> Self {
        Self {
            base: ChannelBase::new(id, "serverStats"),
            sender: OnceLock::new(),
        }
    }
}

impl_channel!(ServerStatsChannel, "serverStats");

pub struct DriveChannel {
    base: ChannelBase,
    #[allow(dead_code)]
    user_id: Option<String>,
    sender: OnceLock<UnboundedSender<ChannelMessage>>,
}

impl DriveChannel {
    pub fn new(id: String, user_id: Option<String>) -> Self {
        Self {
            base: ChannelBase::new(id, "drive"),
            user_id,
            sender: OnceLock::new(),
        }
    }
}

impl_channel!(DriveChannel, "drive");

pub struct ApLogChannel {
    base: ChannelBase,
    sender: OnceLock<UnboundedSender<ChannelMessage>>,
}

impl ApLogChannel {
    pub fn new(id: String) -> Self {
        Self {
            base: ChannelBase::new(id, "apLog"),
            sender: OnceLock::new(),
        }
    }
}

impl_channel!(ApLogChannel, "apLog");

pub struct UserListChannel {
    base: ChannelBase,
    #[allow(dead_code)]
    list_id: String,
    #[allow(dead_code)]
    user_id: Option<String>,
    sender: OnceLock<UnboundedSender<ChannelMessage>>,
}

impl UserListChannel {
    pub fn new(id: String, list_id: String, user_id: Option<String>) -> Self {
        Self {
            base: ChannelBase::new(id, "userList"),
            list_id,
            user_id,
            sender: OnceLock::new(),
        }
    }
}

impl_channel!(UserListChannel, "userList");
