use once_cell::sync::Lazy;
use regex::Regex;

static HASHTAG_SIMPLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#([\w぀-ゟ゠-ヿ一-龯가-힯]+)").unwrap());

pub fn extract_hashtags(text: &str) -> Vec<String> {
    let mut hashtags = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for mat in HASHTAG_SIMPLE_REGEX.find_iter(text) {
        if let Some(caps) = HASHTAG_SIMPLE_REGEX.captures(mat.as_str()) {
            let hashtag = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            if hashtag.is_empty() || hashtag.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let normalized = hashtag.to_lowercase();
            if seen.insert(normalized.clone()) {
                hashtags.push(normalized);
            }
        }
    }
    hashtags
}

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

pub fn is_valid_hashtag(tag: &str) -> bool {
    if tag.is_empty() {
        return false;
    }
    if tag.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    tag.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || is_japanese_character(c))
}

fn is_japanese_character(c: char) -> bool {
    matches!(c as u32,
        0x3040..=0x309F | 0x30A0..=0x30FF | 0x4E00..=0x9FAF |
        0xFF10..=0xFF19 | 0xFF21..=0xFF3A | 0xFF41..=0xFF5A
    )
}

pub fn normalize_hashtag(tag: &str) -> String {
    tag.trim().to_lowercase()
}
