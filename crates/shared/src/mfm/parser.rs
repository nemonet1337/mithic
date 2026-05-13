use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MfmNode {
    Text(String),
    Mention(String),
    Hashtag(String),
    Url(String),
    Bold(Vec<MfmNode>),
    Italic(Vec<MfmNode>),
    Emoji(String),
    InlineCode(String),
    LineBreak,
}

pub fn parse(input: &str) -> Vec<MfmNode> {
    let mut nodes = Vec::new();
    let mut buffer = String::new();
    let mut chars = input.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch == '\n' {
            flush_text(&mut buffer, &mut nodes);
            nodes.push(MfmNode::LineBreak);
            continue;
        }

        let rest = &input[idx..];

        if let Some((token, consumed)) = read_wrapped(rest, "**", "**") {
            flush_text(&mut buffer, &mut nodes);
            nodes.push(MfmNode::Bold(parse(token)));
            skip_chars(&mut chars, consumed.saturating_sub(ch.len_utf8()));
            continue;
        }

        if let Some((token, consumed)) = read_wrapped(rest, "*", "*") {
            flush_text(&mut buffer, &mut nodes);
            nodes.push(MfmNode::Italic(parse(token)));
            skip_chars(&mut chars, consumed.saturating_sub(ch.len_utf8()));
            continue;
        }

        if let Some((token, consumed)) = read_wrapped(rest, "`", "`") {
            flush_text(&mut buffer, &mut nodes);
            nodes.push(MfmNode::InlineCode(token.to_string()));
            skip_chars(&mut chars, consumed.saturating_sub(ch.len_utf8()));
            continue;
        }

        if ch == ':' {
            if let Some((name, consumed)) = read_custom_emoji(rest) {
                flush_text(&mut buffer, &mut nodes);
                nodes.push(MfmNode::Emoji(name.to_string()));
                skip_chars(&mut chars, consumed.saturating_sub(ch.len_utf8()));
                continue;
            }
        }

        if ch == '#' {
            if let Some((tag, consumed)) = read_word(&input[idx + ch.len_utf8()..], true) {
                flush_text(&mut buffer, &mut nodes);
                nodes.push(MfmNode::Hashtag(tag.to_string()));
                skip_chars(&mut chars, consumed);
                continue;
            }
        }

        if ch == '@' {
            if let Some((mention, consumed)) = read_mention(&input[idx + ch.len_utf8()..]) {
                flush_text(&mut buffer, &mut nodes);
                nodes.push(MfmNode::Mention(mention.to_string()));
                skip_chars(&mut chars, consumed);
                continue;
            }
        }

        if rest.starts_with("http://") || rest.starts_with("https://") {
            let (url, consumed) = read_url(rest);
            flush_text(&mut buffer, &mut nodes);
            nodes.push(MfmNode::Url(url.to_string()));
            skip_chars(&mut chars, consumed.saturating_sub(ch.len_utf8()));
            continue;
        }

        buffer.push(ch);
    }

    flush_text(&mut buffer, &mut nodes);
    nodes
}

fn flush_text(buffer: &mut String, nodes: &mut Vec<MfmNode>) {
    if !buffer.is_empty() {
        nodes.push(MfmNode::Text(std::mem::take(buffer)));
    }
}

fn skip_chars(iter: &mut std::iter::Peekable<std::str::CharIndices<'_>>, mut bytes: usize) {
    while bytes > 0 {
        if let Some((_, ch)) = iter.next() {
            bytes = bytes.saturating_sub(ch.len_utf8());
        } else {
            break;
        }
    }
}

fn read_wrapped<'a>(input: &'a str, open: &str, close: &str) -> Option<(&'a str, usize)> {
    if !input.starts_with(open) {
        return None;
    }
    let start = open.len();
    let end = input[start..].find(close)? + start;
    if end == start {
        return None;
    }
    Some((&input[start..end], end + close.len()))
}

fn read_custom_emoji(input: &str) -> Option<(&str, usize)> {
    let end = input.get(1..)?.find(':')? + 1;
    let name = &input[1..end];
    if name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'))
    {
        Some((name, end + 1))
    } else {
        None
    }
}

fn read_word(input: &str, allow_unicode: bool) -> Option<(&str, usize)> {
    let end = input
        .char_indices()
        .take_while(|(_, c)| {
            c.is_ascii_alphanumeric()
                || *c == '_'
                || (allow_unicode && !c.is_ascii() && !c.is_whitespace())
        })
        .map(|(idx, c)| idx + c.len_utf8())
        .last()?;
    Some((&input[..end], end))
}

fn read_mention(input: &str) -> Option<(&str, usize)> {
    let end = input
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_alphanumeric() || matches!(*c, '_' | '-' | '.' | '@'))
        .map(|(idx, c)| idx + c.len_utf8())
        .last()?;
    Some((&input[..end], end))
}

fn read_url(input: &str) -> (&str, usize) {
    let end = input
        .char_indices()
        .take_while(|(_, c)| !c.is_whitespace() && !matches!(*c, '"' | '<' | '>'))
        .map(|(idx, c)| idx + c.len_utf8())
        .last()
        .unwrap_or(input.len());
    (&input[..end], end)
}
