use crate::syntax::{SyntaxToken, TokenKind};
use regex::Regex;
use std::sync::OnceLock;

static HEADER: OnceLock<Regex> = OnceLock::new();
static CODE_INLINE: OnceLock<Regex> = OnceLock::new();
static BOLD: OnceLock<Regex> = OnceLock::new();
static ITALIC: OnceLock<Regex> = OnceLock::new();
static LINK: OnceLock<Regex> = OnceLock::new();

pub fn highlight(line: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    // Headers: # Header
    if let Some(m) = HEADER
        .get_or_init(|| Regex::new(r"^#{1,6}\s.*$").unwrap())
        .find(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Keyword,
        });
        return tokens;
    }

    // Inline code `code`
    for m in CODE_INLINE
        .get_or_init(|| Regex::new(r"`[^`]+`").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::String,
        });
    }

    // Bold **text** or __text__
    for m in BOLD
        .get_or_init(|| Regex::new(r"\*\*[^*]+\*\*|__[^_]+__").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Type,
        });
    }

    // Italic *text* or _text_
    for m in ITALIC
        .get_or_init(|| Regex::new(r"\*[^*]+\*|_[^_]+_").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Type,
        });
    }

    // Links [text](url)
    for m in LINK
        .get_or_init(|| Regex::new(r"\[[^\]]*\]\([^)]*\)").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Attribute,
        });
    }

    tokens.sort_by_key(|t| t.start);
    tokens
}
