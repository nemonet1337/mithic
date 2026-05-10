use std::sync::Arc;

use crate::{
    db::{DragonflyClient, SurrealClient},
    models::Actor,
};

/// 認証サービス
#[derive(Debug, Clone)]
pub struct AuthService {
    surreal: Arc<SurrealClient>,
    dragonfly: Arc<DragonflyClient>,
}

impl AuthService {
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal: Arc::new(surreal),
            dragonfly: Arc::new(dragonfly),
        }
    }

    /// アクセストークンをキャッシュに保存
    pub async fn cache_token(&self, token: &str, actor_id: &str, expiry_secs: u64) -> anyhow::Result<()> {
        let key = format!("token:{}", token);
        redis::cmd("SETEX")
            .arg(&key)
            .arg(expiry_secs)
            .arg(actor_id)
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await?;
        Ok(())
    }

    /// キャッシュからアクターIDを取得
    pub async fn get_actor_id_from_token(&self, token: &str) -> anyhow::Result<Option<String>> {
        let key = format!("token:{}", token);
        let actor_id: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.dragonfly.clone())
            .await?;
        Ok(actor_id)
    }

    /// トークンを無効化
    pub async fn revoke_token(&self, token: &str) -> anyhow::Result<()> {
        let key = format!("token:{}", token);
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await?;
        Ok(())
    }
}
