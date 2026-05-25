use once_cell::sync::Lazy;
use regex::Regex;

static CUSTOM_EMOJI_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r":([a-zA-Z0-9_+-]+):").unwrap());

pub fn extract_emojis(text: &str) -> Vec<String> {
    let mut emojis = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for mat in CUSTOM_EMOJI_REGEX.find_iter(text) {
        if let Some(caps) = CUSTOM_EMOJI_REGEX.captures(mat.as_str()) {
            let emoji_name = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            if emoji_name.is_empty() {
                continue;
            }
            if seen.insert(emoji_name.clone()) {
                emojis.push(emoji_name);
            }
        }
    }
    emojis
}

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

pub fn is_valid_emoji_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-')
}

pub fn remove_custom_emojis(text: &str) -> String {
    CUSTOM_EMOJI_REGEX.replace_all(text, "").to_string()
}
