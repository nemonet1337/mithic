use once_cell::sync::Lazy;
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
            format!("{}@{}", username, host)
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

static MENTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"@([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)*)(?:@([a-zA-Z0-9.-]+\.[a-zA-Z]{2,}))?").unwrap()
});

pub fn extract_mentions(text: &str) -> Vec<Mention> {
    let mut mentions = Vec::new();
    for mat in MENTION_REGEX.find_iter(text) {
        if let Some(caps) = MENTION_REGEX.captures(mat.as_str()) {
            let username = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let host = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            if username.is_empty() {
                continue;
            }
            mentions.push(Mention::new(username, host, mat.start(), mat.end()));
        }
    }
    mentions
}

pub fn extract_unique_mentions(text: &str) -> Vec<Mention> {
    let mentions = extract_mentions(text);
    let mut unique = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for mention in mentions {
        let key = mention.acct.clone();
        if seen.insert(key) {
            unique.push(mention);
        }
    }
    unique
}

pub fn extract_local_mentions(text: &str) -> Vec<Mention> {
    extract_mentions(text)
        .into_iter()
        .filter(|m| m.host.is_empty())
        .collect()
}

pub fn extract_remote_mentions(text: &str) -> Vec<Mention> {
    extract_mentions(text)
        .into_iter()
        .filter(|m| !m.host.is_empty())
        .collect()
}
