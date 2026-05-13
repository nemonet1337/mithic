use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

pub type UserPublicKeyId = Ulid;

/// ユーザー公開鍵の独立管理（ActivityPub HTTP Signatures用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPublicKey {
    pub id: UserPublicKeyId,

    /// 鍵の所有者ユーザーID
    pub user_id: ActorId,

    /// ActivityPub key ID (URI)
    pub key_id: String,

    /// PEM形式の公開鍵
    pub public_key_pem: String,

    pub created_at: DateTime<Utc>,
}

impl UserPublicKey {
    pub fn new(user_id: ActorId, key_id: String, public_key_pem: String) -> Self {
        Self {
            id: UserPublicKeyId::new(),
            user_id,
            key_id,
            public_key_pem,
            created_at: Utc::now(),
        }
    }
}
