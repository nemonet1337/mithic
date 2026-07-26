//! プロセス内ストリームイベントバス
//!
//! 投稿・通知の発生を `/api/v1/streaming` の WebSocket 接続へ配信する。
//! ワイヤフォーマットは `shared::StreamEvent` (`type` + `body`)。

use tokio::sync::broadcast;

/// ストリーム配信するイベント
#[derive(Debug, Clone)]
pub enum StreamBroadcast {
    /// 新規ノート (public/home のみ発行される)。全接続へ配信。
    Note(Box<shared::Note>),
    /// 通知。`user_id` 宛の接続のみへ配信。
    Notification {
        user_id: String,
        notification: Box<shared::Notification>,
    },
}

pub type StreamSender = broadcast::Sender<StreamBroadcast>;
pub type StreamReceiver = broadcast::Receiver<StreamBroadcast>;

pub fn channel() -> StreamSender {
    broadcast::channel(1024).0
}
