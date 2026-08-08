use std::sync::LazyLock;

use regex::Regex;

static HASHTAG_SIMPLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#([\w぀-ゟ゠-ヿ一-龯가-힯]+)").unwrap());

pub fn extract_hashtags(text: &str) -> Vec<String> {
    let mut hashtags = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for caps in HASHTAG_SIMPLE_REGEX.captures_iter(text) {
        let hashtag = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        if hashtag.is_empty() || hashtag.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let normalized = hashtag.to_lowercase();
        if seen.insert(normalized.clone()) {
            hashtags.push(normalized);
        }
    }
    hashtags
}
