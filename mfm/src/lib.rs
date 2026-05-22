//! Misskey Flavored Markdown (MFM) Parser
//!
//! MFM is a lightweight markup language used in Misskey.
//! This module provides parsing and rendering capabilities for MFM syntax.

pub mod node;
pub mod parser;

pub use node::{MfmForest, MfmNode};
pub use parser::{parse, parse_plain};

/// MFMパース結果
pub type ParseResult = Result<MfmForest, ParseError>;

/// パースエラー
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// テキストからMFMノードを抽出（簡易版）
pub fn extract(text: &str, node_type: &str) -> Vec<String> {
    let forest = parse(text).unwrap_or_default();
    let mut results = Vec::new();

    for tree in &forest {
        extract_from_node(tree, node_type, &mut results);
    }

    results
}

fn extract_from_node(node: &node::MfmNode, node_type: &str, results: &mut Vec<String>) {
    if node.node_type() == node_type {
        results.push(node.text_content());
    }

    for child in node.children() {
        extract_from_node(child, node_type, results);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mention() {
        let text = "Hello @alice!";
        let forest = parse(text).unwrap();
        assert!(!forest.is_empty());
    }

    #[test]
    fn test_parse_hashtag() {
        let text = "Check out #misskey!";
        let forest = parse(text).unwrap();
        assert!(!forest.is_empty());
    }

    #[test]
    fn test_parse_url() {
        let text = "Visit https://example.com";
        let forest = parse(text).unwrap();
        assert!(!forest.is_empty());
    }
}
