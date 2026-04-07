use crate::syntax::{SyntaxToken, TokenKind};
use regex::Regex;
use std::sync::OnceLock;

static COMMENT: OnceLock<Regex> = OnceLock::new();
static SECTION: OnceLock<Regex> = OnceLock::new();
static KEY: OnceLock<Regex> = OnceLock::new();
static STRING_DQ: OnceLock<Regex> = OnceLock::new();
static STRING_SQ: OnceLock<Regex> = OnceLock::new();
static NUMBER: OnceLock<Regex> = OnceLock::new();
static BOOLEAN: OnceLock<Regex> = OnceLock::new();

pub fn highlight(line: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    // Comments
    if let Some(m) = COMMENT
        .get_or_init(|| Regex::new(r"#.*$").unwrap())
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
    // [section] headers
    for m in SECTION
        .get_or_init(|| Regex::new(r"^\s*\[+[^\]]*\]+").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Keyword,
        });
    }

    // key = value — highlight the key
    for m in KEY
        .get_or_init(|| Regex::new(r"^\s*[\w\-\.]+\s*=").unwrap())
        .find_iter(line)
    {
        // Only color up to but not including the '='
        let key_end = line[m.start()..m.end()]
            .rfind('=')
            .map(|i| m.start() + i)
            .unwrap_or(m.end());
        tokens.push(SyntaxToken {
            start: m.start(),
            end: key_end,
            kind: TokenKind::Type,
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

    // Single-quoted strings
    for m in STRING_SQ
        .get_or_init(|| Regex::new(r"'[^']*'").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::String,
        });
    }

    // Booleans
    for m in BOOLEAN
        .get_or_init(|| Regex::new(r"\b(true|false)\b").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Keyword,
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
