//! Emoji extraction from text
//!
//! Extracts :custom_emoji: patterns and Unicode emojis from text.

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for custom emoji detection
/// Matches: :emoji_name:
static CUSTOM_EMOJI_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r":([a-zA-Z0-9_+-]+):").unwrap()
});

/// Extract custom emoji names from text
///
/// Returns a vector of unique emoji names (without the colons).
///
/// # Examples
///
/// ```
/// use mithic_backend::misc::extract_emojis;
///
/// let emojis = extract_emojis("Hello :custom_emoji:!");
/// assert_eq!(emojis, vec!["custom_emoji"]);
/// ```
pub fn extract_emojis(text: &str) -> Vec<String> {
    let mut emojis = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for mat in CUSTOM_EMOJI_REGEX.find_iter(text) {
        if let Some(caps) = CUSTOM_EMOJI_REGEX.captures(mat.as_str()) {
            let emoji_name = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();

            if emoji_name.is_empty() {
                continue;
            }

            // Deduplicate
            if seen.insert(emoji_name.clone()) {
                emojis.push(emoji_name);
            }
        }
    }

    emojis
}

/// Extract custom emojis with positions
///
/// Returns a vector of (emoji_name, start_position, end_position) tuples.
pub fn extract_emojis_with_positions(text: &str) -> Vec<(String, usize, usize)> {
    let mut emojis = Vec::new();

    for mat in CUSTOM_EMOJI_REGEX.find_iter(text) {
        if let Some(caps) = CUSTOM_EMOJI_REGEX.captures(mat.as_str()) {
            let emoji_name = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();

            if emoji_name.is_empty() {
                continue;
            }

            emojis.push((emoji_name, mat.start(), mat.end()));
        }
    }

    emojis
}

/// Extract emojis without deduplication
pub fn extract_emojis_raw(text: &str) -> Vec<String> {
    CUSTOM_EMOJI_REGEX
        .find_iter(text)
        .filter_map(|mat| {
            CUSTOM_EMOJI_REGEX
                .captures(mat.as_str())
                .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        })
        .filter(|name| !name.is_empty())
        .collect()
}

/// Check if a string is a valid custom emoji name
pub fn is_valid_emoji_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Valid characters: alphanumeric, underscore, plus, hyphen
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-')
}

/// Extract Unicode emojis from text
///
/// Returns a vector of Unicode emoji strings.
pub fn extract_unicode_emojis(text: &str) -> Vec<String> {
    text.chars()
        .filter(|&c| is_emoji_character(c))
        .map(|c| c.to_string())
        .collect()
}

/// Check if a character is an emoji
fn is_emoji_character(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x1F600..=0x1F64F | // Emoticons
        0x1F300..=0x1F5FF | // Misc Symbols and Pictographs
        0x1F680..=0x1F6FF | // Transport and Map
        0x1F700..=0x1F77F | // Alchemical Symbols
        0x1F780..=0x1F7FF | // Geometric Shapes Extended
        0x1F800..=0x1F8FF | // Supplemental Arrows-C
        0x1F900..=0x1F9FF | // Supplemental Symbols and Pictographs
        0x1FA00..=0x1FA6F | // Chess Symbols
        0x1FA70..=0x1FAFF | // Symbols and Pictographs Extended-A
        0x2600..=0x26FF |   // Misc Symbols
        0x2700..=0x27BF |   // Dingbats
        0xFE00..=0xFE0F |   // Variation Selectors
        0x1F1E0..=0x1F1FF | // Flags
        0x2000..=0x200F |   // Zero-width spaces (for emoji sequences)
        0xFEFF            // Zero-width no-break space
    )
}

/// Extract all emojis (both custom and Unicode) with their types
#[derive(Debug, Clone)]
pub enum EmojiType {
    Custom(String),
    Unicode(String),
}

pub fn extract_all_emojis(text: &str) -> Vec<EmojiType> {
    let mut result = Vec::new();

    // Extract custom emojis
    for name in extract_emojis(text) {
        result.push(EmojiType::Custom(name));
    }

    // Extract Unicode emojis
    for emoji in extract_unicode_emojis(text) {
        result.push(EmojiType::Unicode(emoji));
    }

    result
}

/// Count emojis in text
pub fn count_emojis(text: &str) -> usize {
    extract_emojis(text).len() + extract_unicode_emojis(text).len()
}

/// Check if text contains any emojis
pub fn has_emojis(text: &str) -> bool {
    CUSTOM_EMOJI_REGEX.is_match(text) || text.chars().any(|c| is_emoji_character(c))
}

/// Remove custom emojis from text
pub fn remove_custom_emojis(text: &str) -> String {
    CUSTOM_EMOJI_REGEX.replace_all(text, "").to_string()
}

/// Replace custom emojis with their names (without colons)
pub fn replace_emojis_with_names(text: &str) -> String {
    CUSTOM_EMOJI_REGEX
        .replace_all(text, |caps: &regex::Captures| {
            caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string()
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_custom_emoji() {
        let emojis = extract_emojis("Hello :custom_emoji:!");
        assert_eq!(emojis, vec!["custom_emoji"]);
    }

    #[test]
    fn test_extract_multiple_emojis() {
        let emojis = extract_emojis(":emoji1: :emoji2:");
        assert_eq!(emojis, vec!["emoji1", "emoji2"]);
    }

    #[test]
    fn test_extract_dedupes() {
        let emojis = extract_emojis(":emoji: :emoji: :other:");
        assert_eq!(emojis, vec!["emoji", "other"]);
    }

    #[test]
    fn test_extract_with_plus() {
        let emojis = extract_emojis(":emoji+name:");
        assert_eq!(emojis, vec!["emoji+name"]);
    }

    #[test]
    fn test_extract_with_hyphen() {
        let emojis = extract_emojis(":emoji-name:");
        assert_eq!(emojis, vec!["emoji-name"]);
    }

    #[test]
    fn test_extract_empty() {
        let emojis = extract_emojis("Hello world!");
        assert!(emojis.is_empty());
    }

    #[test]
    fn test_is_valid_emoji_name() {
        assert!(is_valid_emoji_name("emoji"));
        assert!(is_valid_emoji_name("emoji_name"));
        assert!(is_valid_emoji_name("emoji+name"));
        assert!(is_valid_emoji_name("emoji-name"));
        assert!(!is_valid_emoji_name(""));
        assert!(!is_valid_emoji_name("emoji name")); // spaces not allowed
    }

    #[test]
    fn test_extract_unicode_emojis() {
        let emojis = extract_unicode_emojis("Hello 😀👍!");
        assert_eq!(emojis.len(), 2);
        assert_eq!(emojis[0], "😀");
        assert_eq!(emojis[1], "👍");
    }

    #[test]
    fn test_has_emojis() {
        assert!(has_emojis(":emoji:"));
        assert!(has_emojis("😀"));
        assert!(!has_emojis("Hello world"));
    }

    #[test]
    fn test_remove_custom_emojis() {
        let text = remove_custom_emojis("Hello :emoji: world!");
        assert_eq!(text, "Hello  world!");
    }

    #[test]
    fn test_extract_with_positions() {
        let emojis = extract_emojis_with_positions("Hello :emoji: world!");
        assert_eq!(emojis.len(), 1);
        assert_eq!(emojis[0], ("emoji".to_string(), 6, 13));
    }
}
