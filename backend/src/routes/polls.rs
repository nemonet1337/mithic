use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use tracing::{error, info, warn};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{
        Note, NoteVisibility, Poll, PollVote, PollResult, PollChoiceResult,
        CreatePollRequest, VotePollRequest,
    },
    state::{AppState, AuthUser},
};

/// 投票を作成（ノート作成時に使用）
pub async fn create_poll(
    state: &AppState,
    note_id: Ulid,
    req: CreatePollRequest,
) -> Result<Poll> {
    let expires_at = req.expires_in.map(|minutes| {
        Utc::now() + chrono::Duration::minutes(minutes as i64)
    });

    let poll = Poll::new(
        note_id,
        req.choices,
        expires_at,
        req.multiple.unwrap_or(false),
    );

    // SurrealDBに保存
    state.surreal()
        .create(("poll", poll.id.to_string()))
        .content(poll.clone())
        .await
        .map_err(|e| {
            error!("Failed to create poll: {}", e);
            AppError::Database(e)
        })?;

    info!("Created poll {} for note {}", poll.id, note_id);
    Ok(poll)
}

/// 投票結果を取得
pub async fn get_poll_result(
    Path(note_id): Path<String>,
    auth_user: Option<axum::Extension<AuthUser>>,
    State(state): State<AppState>,
) -> Result<Json<PollResult>> {
    let note_id = Ulid::from_string(&note_id)
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // Pollを取得
    let poll: Option<Poll> = state.surreal()
        .select(("poll", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let poll = poll.ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;

    // ユーザーの投票を確認（認証済みの場合）
    let voted_choices: Vec<i32> = if let Some(axum::Extension(auth_user)) = auth_user {
        let mut result = state
            .surreal()
            .query("SELECT choice_index FROM poll_vote WHERE poll_id = $poll_id AND actor_id = $actor_id")
            .bind(("poll_id", poll.id.to_string()))
            .bind(("actor_id", auth_user.user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        result.take::<Vec<i32>>(0)
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // 結果を構築
    let total_votes = poll.total_votes();
    let choices: Vec<PollChoiceResult> = poll.choices
        .into_iter()
        .map(|choice| {
            let percentage = if total_votes > 0 {
                (choice.votes as f64 / total_votes as f64) * 100.0
            } else {
                0.0
            };

            PollChoiceResult {
                index: choice.index,
                text: choice.text,
                votes: choice.votes,
                percentage,
                is_voted: voted_choices.contains(&choice.index),
            }
        })
        .collect();

    Ok(Json(PollResult {
        id: poll.id.to_string(),
        note_id: poll.note_id.to_string(),
        multiple: poll.multiple,
        expires_at: poll.expires_at,
        is_expired: poll.is_expired(),
        choices,
        total_votes,
    }))
}

/// 投票する
pub async fn vote_poll(
    Path(note_id): Path<String>,
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Json(req): Json<VotePollRequest>,
) -> Result<Json<PollResult>> {
    let note_id = Ulid::from_string(&note_id)
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // Pollを取得
    let poll: Option<Poll> = state.surreal()
        .select(("poll", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let mut poll = poll.ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;

    // 投票可能か確認
    if !poll.can_vote() {
        return Err(AppError::Validation("Poll is expired or closed".to_string()));
    }

    // 既に投票済みか確認
    let existing_votes: Vec<PollVote> = state
        .surreal()
        .query("SELECT * FROM poll_vote WHERE poll_id = $poll_id AND actor_id = $actor_id")
        .bind(("poll_id", poll.id.to_string()))
        .bind(("actor_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .unwrap_or_default();

    if !existing_votes.is_empty() && !poll.multiple {
        return Err(AppError::Validation("Already voted".to_string()));
    }

    // 選択肢の検証と投票処理
    let valid_indices: Vec<i32> = poll.choices.iter().map(|c| c.index).collect();

    for choice_index in &req.choices {
        // 選択肢が有効か確認
        if !valid_indices.contains(choice_index) {
            return Err(AppError::Validation(format!("Invalid choice index: {}", choice_index)));
        }

        // 重複投票チェック（複数選択不可の場合）
        if existing_votes.iter().any(|v| v.choice_index == *choice_index) {
            warn!("Duplicate vote attempt by {} on choice {}", auth_user.user_id, choice_index);
            continue;
        }

        // 投票を作成
        let vote = PollVote::new(poll.id, auth_user.user_id, *choice_index);
        state.surreal()
            .create(("poll_vote", vote.id.to_string()))
            .content(vote)
            .await
            .map_err(|e| {
                error!("Failed to create vote: {}", e);
                AppError::Database(e)
            })?;

        // 投票数を更新
        if let Some(choice) = poll.choices.iter_mut().find(|c| c.index == *choice_index) {
            choice.votes += 1;
        }
    }

    // Pollを更新
    state.surreal()
        .update(("poll", poll.id.to_string()))
        .content(poll.clone())
        .await
        .map_err(|e| AppError::Database(e))?;

    info!("User {} voted on poll {}", auth_user.user_id, poll.id);

    // 更新後の結果を返却
    let total_votes = poll.total_votes();
    let voted_choices: Vec<i32> = req.choices.clone();

    let choices: Vec<PollChoiceResult> = poll.choices
        .into_iter()
        .map(|choice| {
            let percentage = if total_votes > 0 {
                (choice.votes as f64 / total_votes as f64) * 100.0
            } else {
                0.0
            };

            PollChoiceResult {
                index: choice.index,
                text: choice.text,
                votes: choice.votes,
                percentage,
                is_voted: voted_choices.contains(&choice.index),
            }
        })
        .collect();

    Ok(Json(PollResult {
        id: poll.id.to_string(),
        note_id: poll.note_id.to_string(),
        multiple: poll.multiple,
        expires_at: poll.expires_at,
        is_expired: poll.is_expired(),
        choices,
        total_votes,
    }))
}

/// 投票を取り消す
pub async fn unvote_poll(
    Path(note_id): Path<String>,
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let note_id = Ulid::from_string(&note_id)
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;

    // Pollを取得
    let poll: Option<Poll> = state.surreal()
        .select(("poll", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let poll = poll.ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;

    // 投票を削除
    let deleted = state
        .surreal()
        .query("DELETE poll_vote WHERE poll_id = $poll_id AND actor_id = $actor_id RETURN BEFORE")
        .bind(("poll_id", poll.id.to_string()))
        .bind(("actor_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .take::<Vec<PollVote>>(0)
        .unwrap_or_default();

    if deleted.is_empty() {
        return Err(AppError::NotFound("Vote not found".to_string()));
    }

    // 投票数を減算
    for vote in deleted {
        state.surreal()
            .query(r#"
                UPDATE poll SET choices[choice_index].votes = choices[choice_index].votes - 1
                WHERE id = $poll_id AND choices[choice_index].index = $choice_index
            "#)
            .bind(("poll_id", poll.id.to_string()))
            .bind(("choice_index", vote.choice_index))
            .await
            .map_err(|e| AppError::Database(e))?;
    }

    info!("User {} unvoted from poll {}", auth_user.user_id, poll.id);

    Ok(Json(serde_json::json!({"success": true})))
}
