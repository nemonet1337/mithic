use std::sync::Arc;

use tracing::{error, info};

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{ActorId, Note, UserListId},
};

/// タイムラインサービス
#[derive(Debug, Clone)]
pub struct TimelineService {
    surreal: Arc<SurrealClient>,
    dragonfly: Arc<DragonflyClient>,
}

impl TimelineService {
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal: Arc::new(surreal),
            dragonfly: Arc::new(dragonfly),
        }
    }

    /// ホームタイムラインをキャッシュ
    pub async fn cache_home_timeline(
        &self,
        actor_id: &str,
        note: &Note,
    ) -> anyhow::Result<()> {
        let key = format!("timeline:home:{}", actor_id);
        let note_json = serde_json::to_string(note)?;

        // タイムラインに追加（先頭に挿入）
        redis::cmd("LPUSH")
            .arg(&key)
            .arg(&note_json)
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await?;

        // 最大200件に制限
        redis::cmd("LTRIM")
            .arg(&key)
            .arg(0)
            .arg(199)
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await?;

        // TTL設定（1時間）
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(3600)
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await?;

        Ok(())
    }

    /// キャッシュからホームタイムラインを取得
    pub async fn get_cached_home_timeline(
        &self,
        actor_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<Note>> {
        let key = format!("timeline:home:{}", actor_id);

        let notes_json: Vec<String> = redis::cmd("LRANGE")
            .arg(&key)
            .arg(0)
            .arg(limit - 1)
            .query_async(&mut self.dragonfly.clone())
            .await?;

        let mut notes = Vec::new();
        for note_json in notes_json {
            if let Ok(note) = serde_json::from_str::<Note>(&note_json) {
                notes.push(note);
            }
        }

        Ok(notes)
    }

    /// パブリックタイムラインをキャッシュ
    pub async fn cache_public_timeline(&self, note: &Note) -> anyhow::Result<()> {
        let key = "timeline:public";
        let note_json = serde_json::to_string(note)?;

        redis::cmd("LPUSH")
            .arg(key)
            .arg(&note_json)
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await?;

        redis::cmd("LTRIM")
            .arg(key)
            .arg(0)
            .arg(499)
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await?;

        redis::cmd("EXPIRE")
            .arg(key)
            .arg(3600)
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await?;

        Ok(())
    }

    /// キャッシュからパブリックタイムラインを取得
    pub async fn get_cached_public_timeline(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<Note>> {
        let key = "timeline:public";

        let notes_json: Vec<String> = redis::cmd("LRANGE")
            .arg(key)
            .arg(0)
            .arg(limit - 1)
            .query_async(&mut self.dragonfly.clone())
            .await?;

        let mut notes = Vec::new();
        for note_json in notes_json {
            if let Ok(note) = serde_json::from_str::<Note>(&note_json) {
                notes.push(note);
            }
        }

        Ok(notes)
    }

    /// タイムラインを無効化
    pub async fn invalidate_timeline(&self, actor_id: &str) -> anyhow::Result<()> {
        let key = format!("timeline:home:{}", actor_id);
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await?;
        Ok(())
    }

    /// ユーザーリストのタイムラインを取得
    pub async fn get_list_timeline(
        &self,
        user_id: &ActorId,
        list_id: &UserListId,
        limit: i64,
        since_id: Option<crate::models::NoteId>,
        until_id: Option<crate::models::NoteId>,
    ) -> Result<Vec<Note>> {
        // リストが存在するかチェック
        let list: Option<crate::models::UserList> = self
            .surreal
            .query("SELECT * FROM user_list WHERE id = $list_id AND user_id = $user_id")
            .bind(("list_id", list_id.to_string()))
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        let list = list.ok_or_else(|| AppError::NotFound("List not found".to_string()))?;

        // リストメンバーのIDを取得
        let members: Vec<crate::models::UserListMembership> = self
            .surreal
            .query("SELECT * FROM user_list_membership WHERE list_id = $list_id")
            .bind(("list_id", list_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        if members.is_empty() {
            return Ok(vec![]);
        }

        let user_ids: Vec<String> = members
            .into_iter()
            .map(|m| m.member_id.to_string())
            .collect();

        // リストメンバーの投稿を取得（ブロック・ミュート除外）
        let query = r#"
            SELECT note.* as notes 
            FROM note 
            WHERE actor_id IN $user_ids
                AND visibility IN ['public', 'home']
                AND id NOT IN (
                    SELECT <-blocked<-note.id FROM block WHERE out = $user_id
                )
                AND id NOT IN (
                    SELECT <-muted<-note.id FROM mute WHERE out = $user_id
                        AND (expires_at IS NONE OR expires_at > time::now())
                )
                AND actor_id NOT IN (
                    SELECT in FROM block WHERE out = $user_id
                )
                AND actor_id NOT IN (
                    SELECT in FROM mute WHERE out = $user_id
                        AND (expires_at IS NONE OR expires_at > time::now())
                )
            ORDER BY created_at DESC
            LIMIT $limit
        "#;

        let mut builder = self
            .surreal
            .query(query)
            .bind(("user_ids", user_ids))
            .bind(("user_id", user_id.to_string()))
            .bind(("limit", limit));

        if let Some(since) = since_id {
            builder = builder.bind(("since_id", since.to_string()));
        }

        if let Some(until) = until_id {
            builder = builder.bind(("until_id", until.to_string()));
        }

        let notes: Vec<Note> = builder
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(notes)
    }
}
