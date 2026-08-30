use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    pub username: String,
    pub host: String,
    pub acct: String,
    pub start: usize,
    pub end: usize,
}

impl Mention {
    pub fn new(
        username: impl Into<String>,
        host: impl Into<String>,
        start: usize,
        end: usize,
    ) -> Self {
        let username = username.into();
        let host = host.into();
        let acct = if host.is_empty() {
            username.clone()
        } else {
            format!("{username}@{host}")
        };
        Self {
            username,
            host,
            acct,
            start,
            end,
        }
    }
}

static MENTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"@([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)*)(?:@([a-zA-Z0-9.-]+\.[a-zA-Z]{2,}))?").unwrap()
});

pub fn extract_mentions(text: &str) -> Vec<Mention> {
    let mut mentions = Vec::new();
    for caps in MENTION_REGEX.captures_iter(text) {
        let username = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let host = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
        if username.is_empty() {
            continue;
        }
        let full = caps.get(0).unwrap();
        mentions.push(Mention::new(username, host, full.start(), full.end()));
    }
    mentions
}

pub fn extract_local_mentions(text: &str) -> Vec<Mention> {
    extract_mentions(text)
        .into_iter()
        .filter(|m| m.host.is_empty())
        .collect()
}
