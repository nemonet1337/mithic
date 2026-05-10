use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

use super::actor::ActorId;
use super::note::NoteId;

/// 投票ID
pub type PollId = Ulid;

/// 投票モデル
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Poll {
    pub id: PollId,

    /// 紐づくノートID
    pub note_id: NoteId,

    /// 作成日時
    pub created_at: DateTime<Utc>,

    /// 期限日時（nullで無期限）
    pub expires_at: Option<DateTime<Utc>>,

    /// 複数選択可能か
    pub multiple: bool,

    /// 期限後も表示するか
    pub is_archived: bool,

    /// 選択肢
    pub choices: Vec<PollChoice>,
}

impl Poll {
    /// 新しい投票を作成
    pub fn new(
        note_id: NoteId,
        choices: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
        multiple: bool,
    ) -> Self {
        let choices = choices
            .into_iter()
            .enumerate()
            .map(|(index, text)| PollChoice {
                index: index as i32,
                text,
                votes: 0,
            })
            .collect();

        Self {
            id: Ulid::new(),
            note_id,
            created_at: Utc::now(),
            expires_at,
            multiple,
            is_archived: false,
            choices,
        }
    }

    /// 投票を締め切る
    pub fn close(&mut self) {
        self.is_archived = true;
    }

    /// 投票期限が切れているか
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    /// 投票可能か
    pub fn can_vote(&self) -> bool {
        !self.is_expired() && !self.is_archived
    }

    /// 総投票数を計算
    pub fn total_votes(&self) -> i32 {
        self.choices.iter().map(|c| c.votes).sum()
    }
}

/// 投票選択肢
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollChoice {
    /// 選択肢インデックス
    pub index: i32,

    /// 選択肢テキスト
    pub text: String,

    /// 投票数
    pub votes: i32,
}

/// 投票投票（ユーザーが投じた票）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollVote {
    pub id: Ulid,

    /// 投票対象のPoll ID
    pub poll_id: PollId,

    /// 投票したユーザーID
    pub actor_id: ActorId,

    /// 選択した選択肢インデックス
    pub choice_index: i32,

    /// 投票日時
    pub created_at: DateTime<Utc>,
}

impl PollVote {
    pub fn new(poll_id: PollId, actor_id: ActorId, choice_index: i32) -> Self {
        Self {
            id: Ulid::new(),
            poll_id,
            actor_id,
            choice_index,
            created_at: Utc::now(),
        }
    }
}

/// 投票結果（API応答用）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PollResult {
    pub id: String,
    pub note_id: String,
    pub multiple: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_expired: bool,
    pub choices: Vec<PollChoiceResult>,
    pub total_votes: i32,
}

/// 投票選択肢結果（API応答用）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PollChoiceResult {
    pub index: i32,
    pub text: String,
    pub votes: i32,
    pub percentage: f64,
    pub is_voted: bool,
}

/// 投票作成リクエスト
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreatePollRequest {
    /// 選択肢リスト
    #[validate(length(min = 2, max = 10))]
    pub choices: Vec<String>,

    /// 期限（分単位、nullで無期限）
    pub expires_in: Option<i32>,

    /// 複数選択可能か
    pub multiple: Option<bool>,
}

/// 投票APIリクエスト
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VotePollRequest {
    /// 選択する選択肢インデックス（複数選択時はリスト）
    pub choices: Vec<i32>,
}
