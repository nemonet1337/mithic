//! Stream channels
//!
//! Various channel implementations for different streaming purposes.

pub mod admin;
pub mod ap_log;
pub mod drive;
pub mod global_timeline;
pub mod hashtag;
pub mod home_timeline;
pub mod queue_stats;
pub mod server_stats;
pub mod user_list;

pub use admin::AdminChannel;
pub use ap_log::ApLogChannel;
pub use drive::DriveChannel;
pub use global_timeline::GlobalTimelineChannel;
pub use hashtag::HashtagChannel;
pub use home_timeline::HomeTimelineChannel;
pub use queue_stats::QueueStatsChannel;
pub use server_stats::ServerStatsChannel;
pub use user_list::UserListChannel;

use crate::stream::channel::{ChannelBase, ChannelMessage};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared channel state
pub type SharedState = Arc<Mutex<ChannelBase>>;
