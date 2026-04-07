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

    // Line comments — once found, rest of line is comment
    if let Some(m) = COMMENT
        .get_or_init(|| Regex::new(r"//.*$").unwrap())
        .find(line)
    {
        let before = &line[..m.start()];
        add_rust_tokens(before, &mut tokens);
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Comment,
        });
        tokens.sort_by_key(|t| t.start);
        return tokens;
    }

    add_rust_tokens(line, &mut tokens);
    tokens.sort_by_key(|t| t.start);
    tokens
}

fn add_rust_tokens(line: &str, tokens: &mut Vec<SyntaxToken>) {
    // Attributes #[...]
    for m in ATTRIBUTE
        .get_or_init(|| Regex::new(r"#\[.*?\]").unwrap())
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Attribute,
        });
    }

    // String literals (basic, not raw strings)
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
            Regex::new(r"\b(fn|let|mut|pub|use|mod|struct|enum|impl|trait|where|for|in|if|else|match|while|loop|return|break|continue|const|static|type|async|await|dyn|ref|self|Self|super|crate|extern|unsafe|move|box|true|false)\b").unwrap()
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
            Regex::new(r"\b(String|Vec|Option|Result|Box|Arc|Rc|Cell|RefCell|HashMap|HashSet|BTreeMap|BTreeSet|usize|isize|u8|u16|u32|u64|u128|i8|i16|i32|i64|i128|f32|f64|bool|char|str)\b").unwrap()
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
        .get_or_init(|| {
            Regex::new(r"\b\d+(\.\d+)?(u8|u16|u32|u64|usize|i8|i16|i32|i64|isize|f32|f64)?\b")
                .unwrap()
        })
        .find_iter(line)
    {
        tokens.push(SyntaxToken {
            start: m.start(),
            end: m.end(),
            kind: TokenKind::Number,
        });
    }
}
