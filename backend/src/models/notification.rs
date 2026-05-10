use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{actor::ActorId, note::NoteId};

/// 通知ID
pub type NotificationId = Ulid;

/// 通知タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// メンション
    Mention,
    /// リプライ
    Reply,
    /// リノート
    Renote,
    /// 引用リノート
    Quote,
    /// リアクション
    Reaction,
    /// フォロー
    Follow,
    /// フォローリクエスト
    FollowRequest,
    /// フォローリクエスト受理
    FollowRequestAccepted,
    /// 投票終了
    PollEnded,
    /// ユーザー登録（管理者向け）
    UserSignup,
}

/// 通知モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: NotificationId,

    pub created_at: DateTime<Utc>,

    /// 通知タイプ
    pub notification_type: NotificationType,

    /// 通知受信者ID
    pub recipient_id: ActorId,

    /// 通知送信者ID（システム通知の場合はnull）
    pub sender_id: Option<ActorId>,

    /// 関連ノートID
    pub note_id: Option<NoteId>,

    /// リアクション種別（Reaction時）
    pub reaction: Option<String>,

    /// 既読フラグ
    pub is_read: bool,
}

impl Notification {
    /// 新しい通知を作成
    pub fn new(
        notification_type: NotificationType,
        recipient_id: ActorId,
        sender_id: Option<ActorId>,
        note_id: Option<NoteId>,
    ) -> Self {
        Self {
            id: NotificationId::new(),
            created_at: Utc::now(),
            notification_type,
            recipient_id,
            sender_id,
            note_id,
            reaction: None,
            is_read: false,
        }
    }

    /// リアクション通知を作成
    pub fn reaction(
        recipient_id: ActorId,
        sender_id: ActorId,
        note_id: NoteId,
        reaction: String,
    ) -> Self {
        let mut notif = Self::new(
            NotificationType::Reaction,
            recipient_id,
            Some(sender_id),
            Some(note_id),
        );
        notif.reaction = Some(reaction);
        notif
    }

    /// メンション通知を作成
    pub fn mention(recipient_id: ActorId, sender_id: ActorId, note_id: NoteId) -> Self {
        Self::new(
            NotificationType::Mention,
            recipient_id,
            Some(sender_id),
            Some(note_id),
        )
    }

    /// リプライ通知を作成
    pub fn reply(recipient_id: ActorId, sender_id: ActorId, note_id: NoteId) -> Self {
        Self::new(
            NotificationType::Reply,
            recipient_id,
            Some(sender_id),
            Some(note_id),
        )
    }

    /// フォロー通知を作成
    pub fn follow(recipient_id: ActorId, sender_id: ActorId) -> Self {
        Self::new(NotificationType::Follow, recipient_id, Some(sender_id), None)
    }

    /// フォローリクエスト通知を作成
    pub fn follow_request(recipient_id: ActorId, sender_id: ActorId) -> Self {
        Self::new(
            NotificationType::FollowRequest,
            recipient_id,
            Some(sender_id),
            None,
        )
    }

    /// 既読にする
    pub fn mark_as_read(&mut self) {
        self.is_read = true;
    }
}
