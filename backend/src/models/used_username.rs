use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type UsedUsernameId = Ulid;

/// 削除済みユーザー名の再利用防止
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsedUsername {
    pub id: UsedUsernameId,

    /// 使用されたユーザー名（小文字）
    pub username: String,

    /// アカウント削除日時
    pub created_at: DateTime<Utc>,
}

impl UsedUsername {
    pub fn new(username: String) -> Self {
        Self {
            id: UsedUsernameId::new(),
            username: username.to_lowercase(),
            created_at: Utc::now(),
        }
    }
}
