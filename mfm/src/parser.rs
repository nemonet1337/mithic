//! MFM Parser Implementation
//!
//! Uses regex-based parsing for MFM syntax.

use once_cell::sync::Lazy;
use regex::Regex;

use super::ParseError;
use super::node::builder;
use super::node::*;

// Regex patterns for MFM syntax
static MENTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"@([\w\-]+)(?:@([\w.\-]+[\w]))?").unwrap());

static HASHTAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#([\w\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FAF]+)").unwrap());

static EMOJI_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r":([a-zA-Z0-9_+-]+):").unwrap());

static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[\w\-._~:/?#\[\]@!$&'()*+,;=%]+[\w/_~-]|https?://[\w\-._~:/?#\[\]@!$&'()*+,;=%]+").unwrap()
});

static CODE_BLOCK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"```(?:([\w+]+)\n)?(.*?)```").unwrap());

static INLINE_CODE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"`([^`]+)`").unwrap());

static BOLD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*\*\*?([^*]+)\*\*\*?|\*\*([^*]+)\*\*").unwrap());

static ITALIC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<i>(.*?)</i>|(?<!\*)\*([^*]+)\*(?!\*)").unwrap());

static STRIKE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"~~(.+?)~~").unwrap());

static CENTER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<center>(.*?)</center>").unwrap());

static SMALL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<small>(.*?)</small>").unwrap());

static QUOTE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"> ([^\n]*(?:\n> [^\n]*)*)").unwrap());

static MATH_INLINE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\\((.+?)\\\)").unwrap());

static MATH_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\\[([^\]]+)\\\]").unwrap());

static SEARCH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.+?)\s+検索").unwrap());

static LINK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[(.+?)\]\((.+?)\)").unwrap());

static UNICODE_EMOJI_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Basic emoji ranges
    Regex::new(r"[\u{1F600}-\u{1F64F}\u{1F300}-\u{1F5FF}\u{1F680}-\u{1F6FF}\u{1F700}-\u{1F77F}\u{1F780}-\u{1F7FF}\u{1F800}-\u{1F8FF}\u{1F900}-\u{1F9FF}\u{1FA00}-\u{1FA6F}\u{2600}-\u{26FF}\u{2700}-\u{27BF}\u{1F1E0}-\u{1F1FF}]").unwrap()
});

/// MFMテキストをパース
pub fn parse(text: &str) -> Result<MfmForest, ParseError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }

    let mut forest = Vec::new();
    let mut remaining = text;
    let mut position = 0;

    while !remaining.is_empty() {
        // Try to match each pattern in order of priority
        if let Some((node, consumed)) = try_match_patterns(remaining) {
            position += consumed;
            remaining = &text[position..];
            forest.push(node);
        } else {
            // No match found, treat as plain text
            let (text_node, consumed) = take_text(remaining);
            position += consumed;
            remaining = &text[position..];
            if !text_node.is_empty() {
                forest.push(builder::text(text_node));
            }
        }
    }

    normalize(forest)
}

/// プレーンテキストとしてパース（装飾を無視）
pub fn parse_plain(text: &str) -> Result<MfmForest, ParseError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }

    let mut forest = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Only match mentions, hashtags, URLs, and emojis
        if let Some((node, consumed)) = try_match_plain_patterns(remaining) {
            remaining = &remaining[consumed..];
            forest.push(node);
        } else {
            let (text_node, consumed) = take_text(remaining);
            remaining = &remaining[consumed..];
            if !text_node.is_empty() {
                forest.push(builder::text(text_node));
            }
        }
    }

    Ok(forest)
}

/// パターンを順に試行
fn try_match_patterns(text: &str) -> Option<(MfmNode, usize)> {
    // Priority order: code blocks, inline patterns, then text formatting

    // Code blocks (highest priority)
    if let Some(mat) = CODE_BLOCK_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = CODE_BLOCK_REGEX
                .captures(&text[mat.start()..mat.end()])
                .unwrap();
            let lang = caps.get(1).map(|m| m.as_str().to_string());
            let code = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            return Some((builder::code_block(code, lang), mat.end()));
        }
    }

    // Quote
    if let Some(mat) = QUOTE_REGEX.find(text) {
        if mat.start() == 0 {
            let content = mat.as_str();
            // Remove "> " prefixes and parse content
            let lines: Vec<&str> = content.lines().collect();
            let cleaned = lines
                .iter()
                .map(|line| line.strip_prefix("> ").unwrap_or(line))
                .collect::<Vec<_>>()
                .join("\n");
            let children = parse(&cleaned).unwrap_or_default();
            return Some((builder::quote(children), mat.end()));
        }
    }

    // Math block
    if let Some(mat) = MATH_BLOCK_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = MATH_BLOCK_REGEX.captures(mat.as_str()).unwrap();
            let formula = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            return Some((builder::math_block(formula), mat.end()));
        }
    }

    // Center
    if let Some(mat) = CENTER_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = CENTER_REGEX.captures(mat.as_str()).unwrap();
            let content = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let children = parse(content).unwrap_or_default();
            return Some((builder::center(children), mat.end()));
        }
    }

    // Small
    if let Some(mat) = SMALL_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = SMALL_REGEX.captures(mat.as_str()).unwrap();
            let content = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let children = parse(content).unwrap_or_default();
            return Some((builder::small(children), mat.end()));
        }
    }

    // Bold (**text** or ***text***)
    if let Some(mat) = BOLD_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = BOLD_REGEX.captures(mat.as_str()).unwrap();
            let content = caps
                .get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str())
                .unwrap_or("");
            let children = parse(content).unwrap_or_default();
            return Some((builder::bold(children), mat.end()));
        }
    }

    // Italic (<i>text</i> or *text*)
    if let Some(mat) = ITALIC_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = ITALIC_REGEX.captures(mat.as_str()).unwrap();
            let content = caps
                .get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str())
                .unwrap_or("");
            let children = parse(content).unwrap_or_default();
            return Some((builder::italic(children), mat.end()));
        }
    }

    // Strike (~~text~~)
    if let Some(mat) = STRIKE_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = STRIKE_REGEX.captures(mat.as_str()).unwrap();
            let content = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let children = parse(content).unwrap_or_default();
            return Some((builder::strike(children), mat.end()));
        }
    }

    // Inline code
    if let Some(mat) = INLINE_CODE_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = INLINE_CODE_REGEX.captures(mat.as_str()).unwrap();
            let code = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            return Some((builder::inline_code(code), mat.end()));
        }
    }

    // Math inline
    if let Some(mat) = MATH_INLINE_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = MATH_INLINE_REGEX.captures(mat.as_str()).unwrap();
            let formula = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            return Some((builder::math_inline(formula), mat.end()));
        }
    }

    // Link [text](url)
    if let Some(mat) = LINK_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = LINK_REGEX.captures(mat.as_str()).unwrap();
            let link_text = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let url = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            let children = parse(link_text).unwrap_or_default();
            return Some((builder::link(url, children), mat.end()));
        }
    }

    // Try plain patterns
    try_match_plain_patterns(text)
}

/// プレーンテキスト用パターン（メンション、ハッシュタグ、URL、絵文字）
fn try_match_plain_patterns(text: &str) -> Option<(MfmNode, usize)> {
    // URL (check before mention to avoid @ in URLs being captured)
    if let Some(mat) = URL_REGEX.find(text) {
        if mat.start() == 0 {
            return Some((builder::url(mat.as_str().to_string()), mat.end()));
        }
    }

    // Mention (@username or @username@host)
    if let Some(mat) = MENTION_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = MENTION_REGEX.captures(mat.as_str()).unwrap();
            let username = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let host = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            return Some((builder::mention(username, host), mat.end()));
        }
    }

    // Hashtag (#tag)
    if let Some(mat) = HASHTAG_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = HASHTAG_REGEX.captures(mat.as_str()).unwrap();
            let hashtag = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            return Some((builder::hashtag(hashtag), mat.end()));
        }
    }

    // Emoji (:name:)
    if let Some(mat) = EMOJI_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = EMOJI_REGEX.captures(mat.as_str()).unwrap();
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            return Some((builder::emoji(name), mat.end()));
        }
    }

    // Unicode emoji
    if let Some(mat) = UNICODE_EMOJI_REGEX.find(text) {
        if mat.start() == 0 {
            return Some((builder::unicode_emoji(mat.as_str().to_string()), mat.end()));
        }
    }

    // Search (query 検索)
    if let Some(mat) = SEARCH_REGEX.find(text) {
        if mat.start() == 0 {
            let caps = SEARCH_REGEX.captures(mat.as_str()).unwrap();
            let query = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let content = mat.as_str().to_string();
            return Some((builder::search(query, content), mat.end()));
        }
    }

    None
}

/// テキストを切り出し（次の特殊パターンまで）
fn take_text(text: &str) -> (String, usize) {
    let mut end = text.len();

    // Find the earliest special character
    for pattern in [
        "@", "#", ":", "*", "`", "~", "<", "[", "\\", ">", "http", "検索",
    ] {
        if let Some(pos) = text.find(pattern) {
            if pos < end && pos > 0 {
                end = pos;
            }
        }
    }

    // Check for Unicode emoji
    if let Some(mat) = UNICODE_EMOJI_REGEX.find(text) {
        if mat.start() > 0 && mat.start() < end {
            end = mat.start();
        }
    }

    (text[..end].to_string(), end)
}

/// パース結果を正規化（隣接するテキストノードを結合等）
fn normalize(mut forest: MfmForest) -> Result<MfmForest, ParseError> {
    if forest.is_empty() {
        return Ok(forest);
    }

    let mut result = Vec::new();
    let mut current_text = String::new();

    for node in forest {
        match node {
            MfmNode::Text(t) => {
                current_text.push_str(&t.text);
            }
            _ => {
                if !current_text.is_empty() {
                    result.push(builder::text(current_text.clone()));
                    current_text.clear();
                }
                result.push(node);
            }
        }
    }

    if !current_text.is_empty() {
        result.push(builder::text(current_text));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mention() {
        let forest = parse("@alice").unwrap();
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].node_type(), "mention");
    }

    #[test]
    fn test_parse_hashtag() {
        let forest = parse("#misskey").unwrap();
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].node_type(), "hashtag");
    }

    #[test]
    fn test_parse_url() {
        let forest = parse("https://example.com").unwrap();
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].node_type(), "url");
    }

    #[test]
    fn test_parse_emoji() {
        let forest = parse(":custom_emoji:").unwrap();
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].node_type(), "emoji");
    }

    #[test]
    fn test_parse_bold() {
        let forest = parse("**bold text**").unwrap();
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].node_type(), "bold");
    }

    #[test]
    fn test_parse_mixed() {
        let forest = parse("Hello @alice, check out #misskey!").unwrap();
        assert!(forest.len() >= 4); // text + mention + text + hashtag + text
    }
}
