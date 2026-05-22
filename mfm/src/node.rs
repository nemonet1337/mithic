//! MFM Node Types
//!
//! Defines all node types that can appear in MFM parsed content.

use serde::{Deserialize, Serialize};

/// MFMノードの種類
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MfmNode {
    /// テキストノード
    Text(TextNode),
    /// メンションノード
    Mention(MentionNode),
    /// ハッシュタグノード
    Hashtag(HashtagNode),
    /// 絵文字ノード（カスタム絵文字）
    Emoji(EmojiNode),
    /// URLノード
    Url(UrlNode),
    /// 検索ノード
    Search(SearchNode),
    /// コードブロック
    CodeBlock(CodeBlockNode),
    /// インラインコード
    InlineCode(InlineCodeNode),
    /// 太字
    Bold(BoldNode),
    /// 斜体
    Italic(ItalicNode),
    /// 打消し線
    Strike(StrikeNode),
    /// 中央寄せ
    Center(CenterNode),
    /// 小さい文字
    Small(SmallNode),
    /// 引用
    Quote(QuoteNode),
    /// 数学式（インライン）
    MathInline(MathInlineNode),
    /// 数学式（ブロック）
    MathBlock(MathBlockNode),
    /// リンク
    Link(LinkNode),
    /// 絵文字コード（Unicode絵文字）
    UnicodeEmoji(UnicodeEmojiNode),
}

/// MFMフォレスト（木のリスト）
pub type MfmForest = Vec<MfmNode>;

impl MfmNode {
    /// ノードタイプ名を取得
    pub fn node_type(&self) -> &'static str {
        match self {
            MfmNode::Text(_) => "text",
            MfmNode::Mention(_) => "mention",
            MfmNode::Hashtag(_) => "hashtag",
            MfmNode::Emoji(_) => "emoji",
            MfmNode::Url(_) => "url",
            MfmNode::Search(_) => "search",
            MfmNode::CodeBlock(_) => "codeBlock",
            MfmNode::InlineCode(_) => "inlineCode",
            MfmNode::Bold(_) => "bold",
            MfmNode::Italic(_) => "italic",
            MfmNode::Strike(_) => "strike",
            MfmNode::Center(_) => "center",
            MfmNode::Small(_) => "small",
            MfmNode::Quote(_) => "quote",
            MfmNode::MathInline(_) => "mathInline",
            MfmNode::MathBlock(_) => "mathBlock",
            MfmNode::Link(_) => "link",
            MfmNode::UnicodeEmoji(_) => "unicodeEmoji",
        }
    }

    /// テキスト内容を取得（リーフノードのみ）
    pub fn text_content(&self) -> String {
        match self {
            MfmNode::Text(n) => n.text.clone(),
            MfmNode::Mention(n) => format!("@{}@{}@", n.username, n.host),
            MfmNode::Hashtag(n) => format!("#{}", n.hashtag),
            MfmNode::Emoji(n) => format!(":{}:", n.name),
            MfmNode::Url(n) => n.url.clone(),
            MfmNode::Search(n) => n.query.clone(),
            MfmNode::CodeBlock(n) => n.code.clone(),
            MfmNode::InlineCode(n) => n.code.clone(),
            MfmNode::UnicodeEmoji(n) => n.emoji.clone(),
            _ => String::new(),
        }
    }

    /// 子ノードを取得
    pub fn children(&self) -> &[MfmNode] {
        match self {
            MfmNode::Bold(n) => &n.children,
            MfmNode::Italic(n) => &n.children,
            MfmNode::Strike(n) => &n.children,
            MfmNode::Center(n) => &n.children,
            MfmNode::Small(n) => &n.children,
            MfmNode::Quote(n) => &n.children,
            MfmNode::Link(n) => &n.children,
            _ => &[],
        }
    }

    /// 子ノードを可変で取得
    pub fn children_mut(&mut self) -> &mut Vec<MfmNode> {
        match self {
            MfmNode::Bold(n) => &mut n.children,
            MfmNode::Italic(n) => &mut n.children,
            MfmNode::Strike(n) => &mut n.children,
            MfmNode::Center(n) => &mut n.children,
            MfmNode::Small(n) => &mut n.children,
            MfmNode::Quote(n) => &mut n.children,
            MfmNode::Link(n) => &mut n.children,
            _ => panic!("Node type {} does not have children", self.node_type()),
        }
    }
}

/// テキストノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextNode {
    pub text: String,
}

impl TextNode {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// メンションノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MentionNode {
    pub username: String,
    pub host: String,
    pub acct: String,
}

impl MentionNode {
    pub fn new(username: impl Into<String>, host: impl Into<String>) -> Self {
        let username = username.into();
        let host = host.into();
        let acct = if host.is_empty() {
            username.clone()
        } else {
            format!("{}@{}", username, host)
        };
        Self {
            username,
            host,
            acct,
        }
    }
}

/// ハッシュタグノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashtagNode {
    pub hashtag: String,
}

impl HashtagNode {
    pub fn new(hashtag: impl Into<String>) -> Self {
        Self {
            hashtag: hashtag.into(),
        }
    }
}

/// 絵文字ノード（カスタム絵文字）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmojiNode {
    pub name: String,
}

impl EmojiNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// URLノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlNode {
    pub url: String,
}

impl UrlNode {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/// 検索ノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchNode {
    pub query: String,
    pub content: String,
}

impl SearchNode {
    pub fn new(query: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            content: content.into(),
        }
    }
}

/// コードブロックノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBlockNode {
    pub code: String,
    pub lang: Option<String>,
}

impl CodeBlockNode {
    pub fn new(code: impl Into<String>, lang: Option<String>) -> Self {
        Self {
            code: code.into(),
            lang,
        }
    }
}

/// インラインコードノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineCodeNode {
    pub code: String,
}

impl InlineCodeNode {
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

/// 太字ノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoldNode {
    pub children: Vec<MfmNode>,
}

impl BoldNode {
    pub fn new(children: Vec<MfmNode>) -> Self {
        Self { children }
    }
}

/// 斜体ノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItalicNode {
    pub children: Vec<MfmNode>,
}

impl ItalicNode {
    pub fn new(children: Vec<MfmNode>) -> Self {
        Self { children }
    }
}

/// 打消し線ノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeNode {
    pub children: Vec<MfmNode>,
}

impl StrikeNode {
    pub fn new(children: Vec<MfmNode>) -> Self {
        Self { children }
    }
}

/// 中央寄せノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CenterNode {
    pub children: Vec<MfmNode>,
}

impl CenterNode {
    pub fn new(children: Vec<MfmNode>) -> Self {
        Self { children }
    }
}

/// 小さい文字ノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmallNode {
    pub children: Vec<MfmNode>,
}

impl SmallNode {
    pub fn new(children: Vec<MfmNode>) -> Self {
        Self { children }
    }
}

/// 引用ノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteNode {
    pub children: Vec<MfmNode>,
}

impl QuoteNode {
    pub fn new(children: Vec<MfmNode>) -> Self {
        Self { children }
    }
}

/// インライン数学式ノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MathInlineNode {
    pub formula: String,
}

impl MathInlineNode {
    pub fn new(formula: impl Into<String>) -> Self {
        Self {
            formula: formula.into(),
        }
    }
}

/// ブロック数学式ノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MathBlockNode {
    pub formula: String,
}

impl MathBlockNode {
    pub fn new(formula: impl Into<String>) -> Self {
        Self {
            formula: formula.into(),
        }
    }
}

/// リンクノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkNode {
    pub url: String,
    pub children: Vec<MfmNode>,
}

impl LinkNode {
    pub fn new(url: impl Into<String>, children: Vec<MfmNode>) -> Self {
        Self {
            url: url.into(),
            children,
        }
    }
}

/// Unicode絵文字ノード
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnicodeEmojiNode {
    pub emoji: String,
}

impl UnicodeEmojiNode {
    pub fn new(emoji: impl Into<String>) -> Self {
        Self {
            emoji: emoji.into(),
        }
    }
}

/// MFMノード作成ヘルパー
pub mod builder {
    use super::*;

    pub fn text(text: impl Into<String>) -> MfmNode {
        MfmNode::Text(TextNode::new(text))
    }

    pub fn mention(username: impl Into<String>, host: impl Into<String>) -> MfmNode {
        MfmNode::Mention(MentionNode::new(username, host))
    }

    pub fn hashtag(tag: impl Into<String>) -> MfmNode {
        MfmNode::Hashtag(HashtagNode::new(tag))
    }

    pub fn emoji(name: impl Into<String>) -> MfmNode {
        MfmNode::Emoji(EmojiNode::new(name))
    }

    pub fn url(url: impl Into<String>) -> MfmNode {
        MfmNode::Url(UrlNode::new(url))
    }

    pub fn search(query: impl Into<String>, content: impl Into<String>) -> MfmNode {
        MfmNode::Search(SearchNode::new(query, content))
    }

    pub fn code_block(code: impl Into<String>, lang: Option<String>) -> MfmNode {
        MfmNode::CodeBlock(CodeBlockNode::new(code, lang))
    }

    pub fn inline_code(code: impl Into<String>) -> MfmNode {
        MfmNode::InlineCode(InlineCodeNode::new(code))
    }

    pub fn bold(children: Vec<MfmNode>) -> MfmNode {
        MfmNode::Bold(BoldNode::new(children))
    }

    pub fn italic(children: Vec<MfmNode>) -> MfmNode {
        MfmNode::Italic(ItalicNode::new(children))
    }

    pub fn strike(children: Vec<MfmNode>) -> MfmNode {
        MfmNode::Strike(StrikeNode::new(children))
    }

    pub fn center(children: Vec<MfmNode>) -> MfmNode {
        MfmNode::Center(CenterNode::new(children))
    }

    pub fn small(children: Vec<MfmNode>) -> MfmNode {
        MfmNode::Small(SmallNode::new(children))
    }

    pub fn quote(children: Vec<MfmNode>) -> MfmNode {
        MfmNode::Quote(QuoteNode::new(children))
    }

    pub fn math_inline(formula: impl Into<String>) -> MfmNode {
        MfmNode::MathInline(MathInlineNode::new(formula))
    }

    pub fn math_block(formula: impl Into<String>) -> MfmNode {
        MfmNode::MathBlock(MathBlockNode::new(formula))
    }

    pub fn link(url: impl Into<String>, children: Vec<MfmNode>) -> MfmNode {
        MfmNode::Link(LinkNode::new(url, children))
    }

    pub fn unicode_emoji(emoji: impl Into<String>) -> MfmNode {
        MfmNode::UnicodeEmoji(UnicodeEmojiNode::new(emoji))
    }
}
