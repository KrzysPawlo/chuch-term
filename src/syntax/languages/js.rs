use crate::syntax::{SyntaxToken, TokenKind};
use regex::Regex;
use std::sync::OnceLock;

static COMMENT_LINE: OnceLock<Regex> = OnceLock::new();
static STRING_DQ: OnceLock<Regex> = OnceLock::new();
static STRING_SQ: OnceLock<Regex> = OnceLock::new();
static STRING_BT: OnceLock<Regex> = OnceLock::new();
static KEYWORDS: OnceLock<Regex> = OnceLock::new();
static NUMBER: OnceLock<Regex> = OnceLock::new();

pub fn highlight(line: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    if let Some(m) = COMMENT_LINE
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
    // Template literals
    for m in STRING_BT
        .get_or_init(|| Regex::new(r"`(?:[^`\\]|\\.)*`").unwrap())
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
            Regex::new(r"\b(function|const|let|var|return|if|else|for|while|do|switch|case|break|continue|import|export|default|class|extends|new|this|typeof|instanceof|async|await|try|catch|finally|throw|true|false|null|undefined)\b").unwrap()
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
