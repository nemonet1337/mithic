use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

/// アクターID (ULID形式)
pub type ActorId = Ulid;

/// アクター種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActorType {
    Local,
    Remote,
}

/// アクターモデル（ユーザー/アクター）
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    pub id: ActorId,

    pub created_at: DateTime<Utc>,

    pub updated_at: Option<DateTime<Utc>>,

    #[validate(length(min = 1, max = 128))]
    pub username: String,

    /// 小文字化されたユーザー名（検索用）
    pub username_lower: String,

    #[validate(length(min = 1, max = 128))]
    pub name: Option<String>,

    /// 自己紹介
    #[validate(length(max = 4096))]
pub bio: Option<String>,

    pub followers_count: i32,

    pub following_count: i32,

    pub notes_count: i32,

    /// アバターURL
    #[validate(length(max = 512))]
    pub avatar_url: Option<String>,

    /// バナーURL
    #[validate(length(max = 512))]
    pub banner_url: Option<String>,

    /// 凍結状態
    pub is_suspended: bool,

    /// ロック状態（フォロー承認制）
    pub is_locked: bool,

    /// Botフラグ
    pub is_bot: bool,

    /// 管理者フラグ
    pub is_admin: bool,

    /// 所属ホスト（nullの場合はローカルユーザー）
    #[validate(length(max = 128))]
    pub host: Option<String>,

    /// ActivityPub inbox URL
    #[validate(length(max = 512))]
    pub inbox: Option<String>,

    /// ActivityPub sharedInbox URL
    #[validate(length(max = 512))]
    pub shared_inbox: Option<String>,

    /// ActivityPub featured URL
    #[validate(length(max = 512))]
    pub featured: Option<String>,

    /// ActivityPub URI (Actor ID)
    #[validate(length(max = 512))]
    pub uri: Option<String>,

    /// ActivityPub公開鍵
    pub public_key: Option<String>,

    /// ActivityPub秘密鍵（ローカルアクターのみ）
    pub private_key: Option<String>,

    /// アクセストークン（ローカルユーザー認証用）
    pub token: Option<String>,

    /// パスワードハッシュ（ローカルユーザー）
    pub password_hash: Option<String>,

    /// メールアドレス（ローカルユーザー）
    pub email: Option<String>,
}

impl Actor {
    /// 新しいローカルアクターを作成
    pub fn new_local(username: String, name: Option<String>) -> Self {
        let now = Utc::now();
        let username_lower = username.to_lowercase();

        Self {
            id: ActorId::new(),
            created_at: now,
            updated_at: None,
            username,
            username_lower,
            name,
            bio: None,
            followers_count: 0,
            following_count: 0,
            notes_count: 0,
            avatar_url: None,
            banner_url: None,
            is_suspended: false,
            is_locked: false,
            is_bot: false,
            is_admin: false,
            host: None,
            inbox: None,
            shared_inbox: None,
            featured: None,
            uri: None,
            public_key: None,
            private_key: None,
            token: None,
            password_hash: None,
            email: None,
        }
    }

    /// ローカルアクターかどうか
    pub fn is_local(&self) -> bool {
        self.host.is_none()
    }

    /// リモートアクターかどうか
    pub fn is_remote(&self) -> bool {
        self.host.is_some()
    }

    /// Actor URI を生成
    pub fn actor_uri(&self, instance_url: &str) -> String {
        format!("{}/users/{}", instance_url, self.username)
    }

    /// Inbox URL を生成
    pub fn inbox_url(&self, instance_url: &str) -> String {
        format!("{}/users/{}/inbox", instance_url, self.username)
    }

    /// Outbox URL を生成
    pub fn outbox_url(&self, instance_url: &str) -> String {
        format!("{}/users/{}/outbox", instance_url, self.username)
    }
}

/// ローカルアクター
pub type LocalActor = Actor;

/// リモートアクター
pub type RemoteActor = Actor;

/// プロフィール更新リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 128))]
    pub display_name: Option<String>,
    #[validate(length(max = 4096))]
    pub bio: Option<String>,
    pub avatar_id: Option<Ulid>,
    pub header_id: Option<Ulid>,
    pub is_locked: Option<bool>,
    pub is_bot: Option<bool>,
}
