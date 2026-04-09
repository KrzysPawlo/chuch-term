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
    Text,
    Log,
    Config,
    Plain,
}

pub fn detect_language(file_path: Option<&std::path::Path>) -> Language {
    let file_name = file_path
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let ext = file_path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let ext = ext.to_ascii_lowercase();
    let file_name = file_name.to_ascii_lowercase();

    if matches!(file_name.as_str(), "log" | "logs")
        || file_name.ends_with(".log")
        || file_name.ends_with(".out")
        || file_name.ends_with(".err")
        || is_rotated_log_name(&file_name)
    {
        return Language::Log;
    }

    if file_name == ".env" || file_name.starts_with(".env.") {
        return Language::Config;
    }

    match ext {
        ext if ext == "rs" => Language::Rust,
        ext if ext == "py" => Language::Python,
        ext if matches!(ext.as_str(), "js" | "ts" | "jsx" | "tsx") => Language::JavaScript,
        ext if ext == "go" => Language::Go,
        ext if ext == "toml" => Language::Toml,
        ext if matches!(ext.as_str(), "yaml" | "yml") => Language::Yaml,
        ext if matches!(ext.as_str(), "sh" | "bash" | "zsh") => Language::Shell,
        ext if matches!(ext.as_str(), "md" | "markdown") => Language::Markdown,
        ext if ext == "proto" => Language::Proto,
        ext if matches!(ext.as_str(), "conf" | "cfg" | "ini" | "properties") => Language::Config,
        ext if matches!(ext.as_str(), "log" | "out" | "err") => Language::Log,
        ext if matches!(ext.as_str(), "txt" | "text") => Language::Text,
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
        Language::Text => vec![],
        Language::Log => languages::log::highlight(line),
        Language::Config => languages::config::highlight(line),
        Language::Plain => vec![],
    }
}

pub mod languages;

fn is_rotated_log_name(file_name: &str) -> bool {
    let Some((base, suffix)) = file_name.rsplit_once('.') else {
        return false;
    };
    base.ends_with(".log") && suffix.chars().all(|ch| ch.is_ascii_digit())
}

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

#[cfg(test)]
mod tests {
    use super::{detect_language, Language};
    use std::path::Path;

    #[test]
    fn detects_devops_file_families() {
        assert_eq!(detect_language(Some(Path::new("app.log"))), Language::Log);
        assert_eq!(detect_language(Some(Path::new("app.log.1"))), Language::Log);
        assert_eq!(detect_language(Some(Path::new(".env.production"))), Language::Config);
        assert_eq!(detect_language(Some(Path::new("settings.ini"))), Language::Config);
        assert_eq!(detect_language(Some(Path::new("notes.txt"))), Language::Text);
        assert_eq!(detect_language(Some(Path::new("api.proto"))), Language::Proto);
    }
}
