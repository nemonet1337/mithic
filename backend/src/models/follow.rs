use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

/// フォロー関係ID
pub type FollowId = Ulid;

/// フォローモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub id: FollowId,

    pub created_at: DateTime<Utc>,

    /// フォロー元（フォローする側）
    pub follower_id: ActorId,

    /// フォロー先（フォローされる側）
    pub followee_id: ActorId,

    /// リモートフォローの場合のinbox URL
    pub inbox: Option<String>,

    /// リモートフォローの場合のsharedInbox URL
    pub shared_inbox: Option<String>,

    /// 承認状態（承認制アカウント用）
    pub is_accepted: bool,
}

/// フォローリクエストID
pub type FollowRequestId = Ulid;

/// フォローリクエスト（承認制アカウント用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowRequest {
    pub id: FollowRequestId,

    pub created_at: DateTime<Utc>,

    /// リクエスト送信者
    pub follower_id: ActorId,

    /// リクエスト先
    pub followee_id: ActorId,

    /// リクエストメッセージ
    pub request_message: Option<String>,
}
