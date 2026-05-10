use chrono::{DateTime, Utc};
use surrealdb::sql::{self, Thing};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::actor::ActorId,
    state::AppState,
};

/// フォローリクエストID
pub type FollowRequestId = Ulid;

/// フォローリクエストサービス
pub struct FollowRequestService;

impl FollowRequestService {
    /// 自分に届いたフォローリクエストの一覧を取得
    pub async fn list_received_requests(state: &AppState, user_id: ActorId) -> Result<Vec<FollowRequest>> {
        let query = r#"
            SELECT * FROM follow_request
            WHERE followee_id = $user_id
            ORDER BY created_at DESC
        "#;

        let mut result = state
            .surreal()
            .query(query)
            .bind(("user_id", user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let requests: Vec<FollowRequest> = result.take(0).map_err(|e| AppError::Database(e))?;

        Ok(requests)
    }

    /// 自分が送信したフォローリクエストの一覧を取得
    pub async fn list_sent_requests(state: &AppState, user_id: ActorId) -> Result<Vec<FollowRequest>> {
        let query = r#"
            SELECT * FROM follow_request
            WHERE follower_id = $user_id
            ORDER BY created_at DESC
        "#;

        let mut result = state
            .surreal()
            .query(query)
            .bind(("user_id", user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let requests: Vec<FollowRequest> = result.take(0).map_err(|e| AppError::Database(e))?;

        Ok(requests)
    }

    /// フォローリクエストを作成
    pub async fn create_request(
        state: &AppState,
        follower_id: ActorId,
        followee_id: ActorId,
        request_message: Option<String>,
    ) -> Result<FollowRequest> {
        let id = FollowRequestId::new();
        let created_at = Utc::now();

        let query = r#"
            CREATE follow_request:$id SET
                id = $id,
                created_at = $created_at,
                follower_id = $follower_id,
                followee_id = $followee_id,
                request_message = $request_message
        "#;

        let mut result = state
            .surreal()
            .query(query)
            .bind(("id", id.to_string()))
            .bind(("created_at", created_at))
            .bind(("follower_id", follower_id.to_string()))
            .bind(("followee_id", followee_id.to_string()))
            .bind(("request_message", request_message))
            .await
            .map_err(|e| AppError::Database(e))?;

        let request: FollowRequest = result.take(0).map_err(|e| AppError::Database(e))?;

        Ok(request)
    }

    /// フォローリクエストを承認
    pub async fn accept_request(
        state: &AppState,
        followee_id: ActorId,
        follower_id: ActorId,
    ) -> Result<()> {
        // リクエストの存在確認
        let check_query = r#"
            SELECT * FROM follow_request
            WHERE followee_id = $followee_id AND follower_id = $follower_id
        "#;

        let mut check_result = state
            .surreal()
            .query(check_query)
            .bind(("followee_id", followee_id.to_string()))
            .bind(("follower_id", follower_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let request: Option<FollowRequest> = check_result.take(0).map_err(|e| AppError::Database(e))?;

        if request.is_none() {
            return Err(AppError::Validation("Follow request not found".to_string()));
        }

        // フォロー関係を作成
        let follow_query = r#"
            RELATE user:$follower_id->follow->user:$followee_id
            SET created_at = time::now(), is_accepted = true
        "#;

        state
            .surreal()
            .query(follow_query)
            .bind(("follower_id", follower_id.to_string()))
            .bind(("followee_id", followee_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        // フォローリクエストを削除
        let delete_query = r#"
            DELETE follow_request WHERE followee_id = $followee_id AND follower_id = $follower_id
        "#;

        state
            .surreal()
            .query(delete_query)
            .bind(("followee_id", followee_id.to_string()))
            .bind(("follower_id", follower_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    /// フォローリクエストを拒否
    pub async fn reject_request(
        state: &AppState,
        followee_id: ActorId,
        follower_id: ActorId,
    ) -> Result<()> {
        // フォローリクエストを削除
        let query = r#"
            DELETE follow_request WHERE followee_id = $followee_id AND follower_id = $follower_id
        "#;

        state
            .surreal()
            .query(query)
            .bind(("followee_id", followee_id.to_string()))
            .bind(("follower_id", follower_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    /// フォローリクエストをキャンセル
    pub async fn cancel_request(
        state: &AppState,
        follower_id: ActorId,
        followee_id: ActorId,
    ) -> Result<()> {
        // フォローリクエストを削除
        let query = r#"
            DELETE follow_request WHERE follower_id = $follower_id AND followee_id = $followee_id
        "#;

        state
            .surreal()
            .query(query)
            .bind(("follower_id", follower_id.to_string()))
            .bind(("followee_id", followee_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    /// フォローリクエストが存在するか確認
    pub async fn exists(
        state: &AppState,
        follower_id: ActorId,
        followee_id: ActorId,
    ) -> Result<bool> {
        let query = r#"
            SELECT count() FROM follow_request
            WHERE follower_id = $follower_id AND followee_id = $followee_id
        "#;

        let mut result = state
            .surreal()
            .query(query)
            .bind(("follower_id", follower_id.to_string()))
            .bind(("followee_id", followee_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let count: Vec<sql::Value> = result.take(0).map_err(|e| AppError::Database(e))?;

        Ok(count.get(0).and_then(|v| v.as_count()).unwrap_or(0) > 0)
    }
}

/// フォローリクエスト
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowRequest {
    pub id: FollowRequestId,
    pub created_at: DateTime<Utc>,
    pub follower_id: ActorId,
    pub followee_id: ActorId,
    pub request_message: Option<String>,
}
