use std::sync::LazyLock;

use regex::Regex;

static CUSTOM_EMOJI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":([a-zA-Z0-9_+-]+):").unwrap());

pub fn extract_emojis(text: &str) -> Vec<String> {
    let mut emojis = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for caps in CUSTOM_EMOJI_REGEX.captures_iter(text) {
        let emoji_name = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        if emoji_name.is_empty() {
            continue;
        }
        if seen.insert(emoji_name.clone()) {
            emojis.push(emoji_name);
        }
    }
    emojis
}
