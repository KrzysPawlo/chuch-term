use crate::syntax::{SyntaxToken, TokenKind};
use regex::Regex;
use std::sync::OnceLock;

static COMMENT: OnceLock<Regex> = OnceLock::new();
static STRING_DQ: OnceLock<Regex> = OnceLock::new();
static STRING_SQ: OnceLock<Regex> = OnceLock::new();
static KEYWORDS: OnceLock<Regex> = OnceLock::new();
static NUMBER: OnceLock<Regex> = OnceLock::new();
static DECORATOR: OnceLock<Regex> = OnceLock::new();

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
    // Decorators
    for m in DECORATOR
        .get_or_init(|| Regex::new(r"@\w+").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Attribute,
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
        .get_or_init(|| Regex::new(r"'(?:[^'\\]|\\.)*'").unwrap())
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
            Regex::new(r"\b(def|class|import|from|return|if|elif|else|for|while|try|except|finally|with|as|pass|break|continue|lambda|yield|async|await|and|or|not|in|is|True|False|None)\b").unwrap()
        })
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
