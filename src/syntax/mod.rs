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
        Language::Plain => vec![],
    }
}

pub mod languages;
