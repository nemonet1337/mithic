//! Hashtag extraction from text
//!
//! Extracts #hashtag patterns from text.

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for hashtag detection
/// Matches: #tag (alphanumeric and Japanese characters)
static HASHTAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Matches hashtags with alphanumeric and Japanese characters
    // Excludes pure numbers to avoid matching #123 as hashtag
    Regex::new(r"#([\p{L}\p{N}_]+)(?:\p{P}|\p{Z}|\p{C}|$)").unwrap()
});

/// Simpler regex for just finding hashtags
static HASHTAG_SIMPLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"#([\w\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FAF\uAC00-\uD7AF]+)").unwrap()
});

/// Extract hashtags from text
///
/// Returns a vector of unique hashtag strings (without the # prefix, normalized to lowercase).
///
/// # Examples
///
/// ```
/// use mithic_backend::misc::extract_hashtags;
///
/// let hashtags = extract_hashtags("Hello #Misskey and #ActivityPub!");
/// assert_eq!(hashtags, vec!["misskey", "activitypub"]);
/// ```
pub fn extract_hashtags(text: &str) -> Vec<String> {
    let mut hashtags = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for mat in HASHTAG_SIMPLE_REGEX.find_iter(text) {
        if let Some(caps) = HASHTAG_SIMPLE_REGEX.captures(mat.as_str()) {
            let hashtag = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();

            // Skip empty or numeric-only hashtags
            if hashtag.is_empty() || hashtag.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            // Normalize to lowercase
            let normalized = hashtag.to_lowercase();

            // Deduplicate
            if seen.insert(normalized.clone()) {
                hashtags.push(normalized);
            }
        }
    }

    hashtags
}

/// Extract hashtags with positions
///
/// Returns a vector of (hashtag, start_position, end_position) tuples.
pub fn extract_hashtags_with_positions(text: &str) -> Vec<(String, usize, usize)> {
    let mut hashtags = Vec::new();

    for mat in HASHTAG_SIMPLE_REGEX.find_iter(text) {
        if let Some(caps) = HASHTAG_SIMPLE_REGEX.captures(mat.as_str()) {
            let hashtag = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();

            if hashtag.is_empty() || hashtag.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            hashtags.push((hashtag.to_lowercase(), mat.start(), mat.end()));
        }
    }

    hashtags
}

/// Extract hashtags without deduplication
pub fn extract_hashtags_raw(text: &str) -> Vec<String> {
    HASHTAG_SIMPLE_REGEX
        .find_iter(text)
        .filter_map(|mat| {
            HASHTAG_SIMPLE_REGEX
                .captures(mat.as_str())
                .and_then(|caps| caps.get(1).map(|m| m.as_str().to_lowercase()))
        })
        .filter(|tag| !tag.is_empty() && !tag.chars().all(|c| c.is_ascii_digit()))
        .collect()
}

/// Check if a string is a valid hashtag
pub fn is_valid_hashtag(tag: &str) -> bool {
    if tag.is_empty() {
        return false;
    }

    // Must not be pure numbers
    if tag.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Check characters are valid
    tag.chars().all(|c| {
        c.is_ascii_alphanumeric() ||
        c == '_' ||
        is_japanese_character(c)
    })
}

/// Check if character is Japanese
fn is_japanese_character(c: char) -> bool {
    matches!(c as u32,
        0x3040..=0x309F | // Hiragana
        0x30A0..=0x30FF | // Katakana
        0x4E00..=0x9FAF | // CJK Unified Ideographs
        0xFF10..=0xFF19 | // Fullwidth numbers
        0xFF21..=0xFF3A | // Fullwidth uppercase
        0xFF41..=0xFF5A   // Fullwidth lowercase
    )
}

/// Normalize hashtag (lowercase, trim)
pub fn normalize_hashtag(tag: &str) -> String {
    tag.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_hashtag() {
        let hashtags = extract_hashtags("Hello #misskey!");
        assert_eq!(hashtags, vec!["misskey"]);
    }

    #[test]
    fn test_extract_multiple_hashtags() {
        let hashtags = extract_hashtags("#misskey #activitypub #rust");
        assert_eq!(hashtags, vec!["misskey", "activitypub", "rust"]);
    }

    #[test]
    fn test_extract_dedupes() {
        let hashtags = extract_hashtags("#misskey #misskey #activitypub");
        assert_eq!(hashtags, vec!["misskey", "activitypub"]);
    }

    #[test]
    fn test_extract_normalizes_case() {
        let hashtags = extract_hashtags("#Misskey #ACTIVITYPUB");
        assert_eq!(hashtags, vec!["misskey", "activitypub"]);
    }

    #[test]
    fn test_extract_japanese_hashtag() {
        let hashtags = extract_hashtags("#日本語 #テスト");
        assert_eq!(hashtags, vec!["日本語", "テスト"]);
    }

    #[test]
    fn test_extract_korean_hashtag() {
        let hashtags = extract_hashtags("#한국어 #테스트");
        assert_eq!(hashtags, vec!["한국어", "테스트"]);
    }

    #[test]
    fn test_extract_skips_numeric() {
        let hashtags = extract_hashtags("#123 #misskey");
        assert_eq!(hashtags, vec!["misskey"]);
    }

    #[test]
    fn test_extract_empty() {
        let hashtags = extract_hashtags("Hello world!");
        assert!(hashtags.is_empty());
    }

    #[test]
    fn test_is_valid_hashtag() {
        assert!(is_valid_hashtag("misskey"));
        assert!(is_valid_hashtag("activity_pub"));
        assert!(is_valid_hashtag("日本語"));
        assert!(!is_valid_hashtag("123"));
        assert!(!is_valid_hashtag(""));
    }

    #[test]
    fn test_extract_with_positions() {
        let hashtags = extract_hashtags_with_positions("Hello #misskey world!");
        assert_eq!(hashtags.len(), 1);
        assert_eq!(hashtags[0], ("misskey".to_string(), 6, 14));
    }
}
