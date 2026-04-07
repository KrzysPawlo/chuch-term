use crate::syntax::{SyntaxToken, TokenKind};
use regex::Regex;
use std::sync::OnceLock;

static COMMENT: OnceLock<Regex> = OnceLock::new();
static STRING_DQ: OnceLock<Regex> = OnceLock::new();
static STRING_BT: OnceLock<Regex> = OnceLock::new();
static KEYWORDS: OnceLock<Regex> = OnceLock::new();
static TYPES: OnceLock<Regex> = OnceLock::new();
static NUMBER: OnceLock<Regex> = OnceLock::new();

pub fn highlight(line: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    if let Some(m) = COMMENT
        .get_or_init(|| Regex::new(r"//.*$").unwrap())
        .find(line)
    {
        let before = &line[..m.start()];
        add_tokens(before, &mut tokens);
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Comment,
        });
        tokens.sort_by_key(|t| t.start);
        return tokens;
    }

    add_tokens(line, &mut tokens);
    tokens.sort_by_key(|t| t.start);
    tokens
}

fn add_tokens(line: &str, tokens: &mut Vec<SyntaxToken>) {
    // Raw strings (backtick)
    for m in STRING_BT
        .get_or_init(|| Regex::new(r"`[^`]*`").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::String,
        });
    }

    // Double-quoted strings
    for m in STRING_DQ
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
            Regex::new(r"\b(func|package|import|var|const|type|struct|interface|map|chan|go|defer|return|if|else|for|range|switch|case|break|continue|select|true|false|nil)\b").unwrap()
        })
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Keyword,
        });
    }

    // Types
    for m in TYPES
        .get_or_init(|| {
            Regex::new(r"\b(string|int|int8|int16|int32|int64|uint|uint8|uint16|uint32|uint64|float32|float64|bool|byte|rune|error)\b").unwrap()
        })
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Type,
        });
    }

    // Numbers
    for m in NUMBER
        .get_or_init(|| Regex::new(r"\b\d+(\.\d+)?\b").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Number,
        });
    }
}
