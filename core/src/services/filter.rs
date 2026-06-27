use crate::models::actor::Actor;
use crate::models::note::Note;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WordMute {
    pub id: String,
    pub user_id: String,
    pub pattern: String,
    pub created_at: String,
}

pub fn apply_word_mutes(notes: Vec<(Note, Actor)>, mutes: &[String]) -> Vec<(Note, Actor)> {
    notes
        .into_iter()
        .filter(|(note, user)| {
            !mutes.iter().any(|pattern| {
                note.text.as_ref().map(|t| t.contains(pattern)).unwrap_or(false)
                    || user.username.to_lowercase().contains(&pattern.to_lowercase())
            })
        })
        .collect()
}