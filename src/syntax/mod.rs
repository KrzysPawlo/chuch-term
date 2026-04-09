#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    String,
    Comment,
    Number,
    Type,
    Attribute,
}

pub struct SyntaxToken {
    pub start: usize, // byte offset in line
    pub end: usize,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    Go,
    Toml,
    Yaml,
    Shell,
    Markdown,
    Proto,
    Plain,
}

pub fn detect_language(file_path: Option<&std::path::Path>) -> Language {
    let ext = file_path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => Language::Rust,
        "py" => Language::Python,
        "js" | "ts" | "jsx" | "tsx" => Language::JavaScript,
        "go" => Language::Go,
        "toml" => Language::Toml,
        "yaml" | "yml" => Language::Yaml,
        "sh" | "bash" | "zsh" => Language::Shell,
        "md" | "markdown" => Language::Markdown,
        "proto" => Language::Proto,
        _ => Language::Plain,
    }
}

pub fn highlight_line(line: &str, lang: Language) -> Vec<SyntaxToken> {
    match lang {
        Language::Rust => languages::rust::highlight(line),
        Language::Python => languages::python::highlight(line),
        Language::JavaScript => languages::js::highlight(line),
        Language::Go => languages::go::highlight(line),
        Language::Toml => languages::toml::highlight(line),
        Language::Yaml => languages::yaml::highlight(line),
        Language::Shell => languages::shell::highlight(line),
        Language::Markdown => languages::markdown::highlight(line),
        Language::Proto => languages::proto::highlight(line),
        Language::Plain => vec![],
    }
}

pub mod languages;

/// Returns `true` when `line`'s leading whitespace is likely erroneous for the
/// given language.  Only checks YAML, Python, and Proto3 — indentation is
/// syntax-critical there.  For all other languages this always returns `false`.
///
/// Error conditions:
///  - Mixed tabs and spaces in the leading whitespace.
///  - Leading space count is not a multiple of `tab_width`.
pub fn has_indent_error(line: &str, tab_width: u8, lang: Language) -> bool {
    if !matches!(lang, Language::Yaml | Language::Python | Language::Proto) {
        return false;
    }
    let leading: &str = {
        let end = line
            .char_indices()
            .take_while(|(_, c)| *c == ' ' || *c == '\t')
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        &line[..end]
    };
    if leading.is_empty() {
        return false;
    }
    let has_tabs = leading.contains('\t');
    let has_spaces = leading.contains(' ');
    if has_tabs && has_spaces {
        return true; // mixed
    }
    if has_spaces && tab_width > 0 {
        return !leading.len().is_multiple_of(tab_width as usize);
    }
    false
}
