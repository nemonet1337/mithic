use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

use super::actor::ActorId;
use super::note::NoteId;

pub type PollId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Poll {
    pub id: PollId,
    pub note_id: NoteId,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub multiple: bool,
    pub is_archived: bool,
    pub choices: Vec<PollChoice>,
}

impl Poll {
    pub fn new(note_id: NoteId, choices: Vec<String>, expires_at: Option<DateTime<Utc>>, multiple: bool) -> Self {
        let choices = choices.into_iter().enumerate()
            .map(|(index, text)| PollChoice { index: index as i32, text, votes: 0 })
            .collect();
        Self { id: Ulid::new(), note_id, created_at: Utc::now(), expires_at, multiple, is_archived: false, choices }
    }

    pub fn close(&mut self) { self.is_archived = true; }
    pub fn is_expired(&self) -> bool { self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false) }
    pub fn can_vote(&self) -> bool { !self.is_expired() && !self.is_archived }
    pub fn total_votes(&self) -> i32 { self.choices.iter().map(|c| c.votes).sum() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollChoice {
    pub index: i32,
    pub text: String,
    pub votes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollVote {
    pub id: Ulid,
    pub poll_id: PollId,
    pub actor_id: ActorId,
    pub choice_index: i32,
    pub created_at: DateTime<Utc>,
}

impl PollVote {
    pub fn new(poll_id: PollId, actor_id: ActorId, choice_index: i32) -> Self {
        Self { id: Ulid::new(), poll_id, actor_id, choice_index, created_at: Utc::now() }
    }
}

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PollChoiceResult {
    pub index: i32,
    pub text: String,
    pub votes: i32,
    pub percentage: f64,
    pub is_voted: bool,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreatePollRequest {
    #[validate(length(min = 2, max = 10))]
    pub choices: Vec<String>,
    pub expires_in: Option<i32>,
    pub multiple: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VotePollRequest {
    pub choices: Vec<i32>,
}
