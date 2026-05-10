//! Mention extraction from text
//!
//! Extracts @username and @username@host mentions from text.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Mention information
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
    pub fn new(username: impl Into<String>, host: impl Into<String>, start: usize, end: usize) -> Self {
        let username = username.into();
        let host = host.into();
        let acct = if host.is_empty() {
            username.clone()
        } else {
            format!("{}@{}", username, host)
        };
        Self { username, host, acct, start, end }
    }
}

/// Regex for mention detection
/// Matches: @username or @username@host
static MENTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"@([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)*)(?:@([a-zA-Z0-9.-]+\.[a-zA-Z]{2,}))?").unwrap()
});

/// Extract mentions from text
///
/// # Examples
///
/// ```
/// use mithic_backend::misc::extract_mentions;
///
/// let mentions = extract_mentions("Hello @alice and @bob@example.com!");
/// assert_eq!(mentions.len(), 2);
/// assert_eq!(mentions[0].username, "alice");
/// assert_eq!(mentions[1].username, "bob");
/// assert_eq!(mentions[1].host, "example.com");
/// ```
pub fn extract_mentions(text: &str) -> Vec<Mention> {
    let mut mentions = Vec::new();

    for mat in MENTION_REGEX.find_iter(text) {
        if let Some(caps) = MENTION_REGEX.captures(mat.as_str()) {
            let username = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let host = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();

            // Skip invalid mentions
            if username.is_empty() {
                continue;
            }

            mentions.push(Mention::new(
                username,
                host,
                mat.start(),
                mat.end(),
            ));
        }
    }

    mentions
}

/// Extract unique mentions from text (removes duplicates)
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

/// Extract only local mentions (no host specified)
pub fn extract_local_mentions(text: &str) -> Vec<Mention> {
    extract_mentions(text)
        .into_iter()
        .filter(|m| m.host.is_empty())
        .collect()
}

/// Extract only remote mentions (host specified)
pub fn extract_remote_mentions(text: &str) -> Vec<Mention> {
    extract_mentions(text)
        .into_iter()
        .filter(|m| !m.host.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_local_mention() {
        let mentions = extract_mentions("Hello @alice!");
        assert_eq!(mentions.len(), 1);
        assert_eq!(mentions[0].username, "alice");
        assert_eq!(mentions[0].host, "");
        assert_eq!(mentions[0].acct, "alice");
    }

    #[test]
    fn test_extract_remote_mention() {
        let mentions = extract_mentions("@bob@example.com");
        assert_eq!(mentions.len(), 1);
        assert_eq!(mentions[0].username, "bob");
        assert_eq!(mentions[0].host, "example.com");
        assert_eq!(mentions[0].acct, "bob@example.com");
    }

    #[test]
    fn test_extract_multiple_mentions() {
        let mentions = extract_mentions("@alice @bob@example.com @charlie");
        assert_eq!(mentions.len(), 3);
    }

    #[test]
    fn test_extract_unique_mentions() {
        let mentions = extract_unique_mentions("@alice @alice @bob");
        assert_eq!(mentions.len(), 2);
    }

    #[test]
    fn test_no_mentions() {
        let mentions = extract_mentions("Hello world!");
        assert!(mentions.is_empty());
    }

    #[test]
    fn test_mention_with_underscore() {
        let mentions = extract_mentions("@user_name");
        assert_eq!(mentions.len(), 1);
        assert_eq!(mentions[0].username, "user_name");
    }

    #[test]
    fn test_mention_in_code_block() {
        // Code blocks should be handled by caller
        let mentions = extract_mentions("`@alice` @bob");
        // Both are extracted - caller should exclude code blocks
        assert_eq!(mentions.len(), 2);
    }
}
