use crate::syntax::{SyntaxToken, TokenKind};
use regex::Regex;
use std::sync::OnceLock;

static COMMENT: OnceLock<Regex> = OnceLock::new();
static STRING: OnceLock<Regex> = OnceLock::new();
static KEYWORDS: OnceLock<Regex> = OnceLock::new();
static TYPES: OnceLock<Regex> = OnceLock::new();
static NUMBER: OnceLock<Regex> = OnceLock::new();
static ATTRIBUTE: OnceLock<Regex> = OnceLock::new();

pub fn highlight(line: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    // Line comments (//)
    if let Some(m) = COMMENT
        .get_or_init(|| Regex::new(r"//.*$").unwrap())
        .find(line)
    {
        let before = &line[..m.start()];
        add_proto_tokens(before, &mut tokens);
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Comment,
        });
        tokens.sort_by_key(|t| t.start);
        return tokens;
    }

    add_proto_tokens(line, &mut tokens);
    tokens.sort_by_key(|t| t.start);
    tokens
}

fn add_proto_tokens(line: &str, tokens: &mut Vec<SyntaxToken>) {
    // String literals
    for m in STRING
        .get_or_init(|| Regex::new(r#""(?:[^"\\]|\\.)*""#).unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::String,
        });
    }

    // Keywords
    for m in KEYWORDS
        .get_or_init(|| {
            Regex::new(r"\b(syntax|package|import|option|message|enum|service|rpc|returns|oneof|repeated|map|reserved|extensions|extend|to|max|stream|weak|public|optional|required)\b")
                .unwrap()
        })
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Keyword,
        });
    }

    // Scalar types
    for m in TYPES
        .get_or_init(|| {
            Regex::new(r"\b(double|float|int32|int64|uint32|uint64|sint32|sint64|fixed32|fixed64|sfixed32|sfixed64|bool|string|bytes)\b")
                .unwrap()
        })
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Type,
        });
    }

    // Numbers (field tags, option values, reserved ranges)
    for m in NUMBER
        .get_or_init(|| Regex::new(r"\b\d+\b").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Number,
        });
    }

    for m in ATTRIBUTE
        .get_or_init(|| Regex::new(r"\[[^\]]+\]|\([^\)]+\)").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Attribute,
        });
    }
}
