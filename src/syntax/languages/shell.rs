use crate::syntax::{SyntaxToken, TokenKind};
use regex::Regex;
use std::sync::OnceLock;

static COMMENT: OnceLock<Regex> = OnceLock::new();
static VARIABLE: OnceLock<Regex> = OnceLock::new();
static STRING_DQ: OnceLock<Regex> = OnceLock::new();
static STRING_SQ: OnceLock<Regex> = OnceLock::new();
static KEYWORDS: OnceLock<Regex> = OnceLock::new();

pub fn highlight(line: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    // Comments
    if let Some(m) = COMMENT
        .get_or_init(|| Regex::new(r"#.*$").unwrap())
        .find(line)
    {
        // Don't mark shebang lines as comment (just comment the # part)
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

    // $VARIABLE references
    for m in VARIABLE
        .get_or_init(|| Regex::new(r"\$\{?\w+\}?").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Type,
        });
    }

    // Keywords
    for m in KEYWORDS
        .get_or_init(|| {
            Regex::new(r"\b(if|then|else|elif|fi|for|do|done|while|case|esac|function|return|echo|export|source|local|shift|read|exit|break|continue)\b").unwrap()
        })
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Keyword,
        });
    }
}
