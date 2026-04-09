use crate::syntax::{SyntaxToken, TokenKind};
use regex::Regex;
use std::sync::OnceLock;

static TIMESTAMP: OnceLock<Regex> = OnceLock::new();
static LEVEL: OnceLock<Regex> = OnceLock::new();
static QUOTED: OnceLock<Regex> = OnceLock::new();
static KEY_VALUE: OnceLock<Regex> = OnceLock::new();
static IPV4: OnceLock<Regex> = OnceLock::new();
static NUMBER: OnceLock<Regex> = OnceLock::new();

pub fn highlight(line: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    for m in TIMESTAMP
        .get_or_init(|| {
            Regex::new(
                r"\b\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:[.,]\d+)?(?:Z|[+-]\d{2}:\d{2})?\b",
            )
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

    for m in LEVEL
        .get_or_init(|| Regex::new(r"\b(TRACE|DEBUG|INFO|WARN|WARNING|ERROR|FATAL)\b").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Keyword,
        });
    }

    for m in QUOTED
        .get_or_init(|| Regex::new(r#""(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'"#).unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::String,
        });
    }

    for m in KEY_VALUE
        .get_or_init(|| Regex::new(r"\b[\w.\-]+(?:=|:)\S+").unwrap())
        .find_iter(line)
    {
        let key_end = line[m.start()..m.end()]
            .find(|ch| ['=', ':'].contains(&ch))
            .map(|idx| m.start() + idx)
            .unwrap_or(m.end());
        tokens.push(SyntaxToken {
            start: m.start(),
            end: key_end,
            kind: TokenKind::Attribute,
        });
    }

    for m in IPV4
        .get_or_init(|| Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Type,
        });
    }

    for m in NUMBER
        .get_or_init(|| Regex::new(r"\b\d+(?:\.\d+)?\b").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Number,
        });
    }

    tokens.sort_by_key(|token| token.start);
    tokens
}
