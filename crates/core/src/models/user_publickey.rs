use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

pub type UserPublicKeyId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPublicKey {
    pub id: UserPublicKeyId,
    pub user_id: ActorId,
    pub key_id: String,
    pub public_key_pem: String,
    pub created_at: DateTime<Utc>,
}

impl UserPublicKey {
    pub fn new(user_id: ActorId, key_id: String, public_key_pem: String) -> Self {
        Self { id: UserPublicKeyId::new(), user_id, key_id, public_key_pem, created_at: Utc::now() }
    }
}
