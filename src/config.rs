use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const DEFAULT_CONFIG_CONTENT: &str = r##"# chuch-term configuration
# Location: ~/.config/chuch-term/config.toml
# Changes are hot-reloaded within 2 seconds.

[editor]
line_numbers = true
relative_numbers = false
syntax_highlight = true

[clipboard]
# "auto" = detect system clipboard, "internal" = never use system clipboard, "osc52" = force OSC-52
strategy = "auto"
"##;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSection {
    pub line_numbers: bool,
    pub relative_numbers: bool,
    pub syntax_highlight: bool,
}

impl Default for EditorSection {
    fn default() -> Self {
        Self {
            line_numbers: true,
            relative_numbers: false,
            syntax_highlight: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSection {
    pub strategy: String,
}

impl Default for ClipboardSection {
    fn default() -> Self {
        Self {
            strategy: "auto".into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorConfig {
    #[serde(default)]
    pub editor: EditorSection,
    #[serde(default)]
    pub clipboard: ClipboardSection,
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("chuch-term").join("config.toml"))
}

pub fn load_config() -> (EditorConfig, Option<String>) {
    let path = match config_path() {
        Some(path) => path,
        None => return (EditorConfig::default(), None),
    };
    load_config_from_path(&path)
}

pub fn load_config_from_path(path: &Path) -> (EditorConfig, Option<String>) {
    if !path.exists() {
        match create_default_config(path) {
            Ok(true) => {
                return (
                    EditorConfig::default(),
                    Some("Config created: ~/.config/chuch-term/config.toml".into()),
                );
            }
            Ok(false) => {}
            Err(err) => {
                return (
                    EditorConfig::default(),
                    Some(format!("Config create error: {err}")),
                );
            }
        }
    }

    match std::fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<EditorConfig>(&content) {
            Ok(cfg) => {
                let (cfg, warn) = validate_config(cfg);
                (cfg, warn)
            }
            Err(err) => (
                EditorConfig::default(),
                Some(format!("Config parse error: {err}")),
            ),
        },
        Err(err) => (
            EditorConfig::default(),
            Some(format!("Config read error: {err}")),
        ),
    }
}

pub fn config_mtime() -> Option<SystemTime> {
    config_path()?.metadata().ok()?.modified().ok()
}

fn validate_config(mut cfg: EditorConfig) -> (EditorConfig, Option<String>) {
    const VALID_STRATEGIES: &[&str] = &["auto", "internal", "osc52"];
    if !VALID_STRATEGIES.contains(&cfg.clipboard.strategy.as_str()) {
        let bad = std::mem::replace(&mut cfg.clipboard.strategy, "auto".to_string());
        return (
            cfg,
            Some(format!(
                "Config: unknown clipboard.strategy {bad:?}, using \"auto\""
            )),
        );
    }
    (cfg, None)
}

fn create_default_config(path: &Path) -> std::io::Result<bool> {
    if path.exists() {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, DEFAULT_CONFIG_CONTENT)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("chuch-term-config-{name}-{nanos}"))
    }

    #[test]
    fn creates_default_config_without_legacy_keys() {
        let root = temp_path("create");
        let path = root.join("config.toml");

        let (config, message) = load_config_from_path(&path);

        assert!(config.editor.line_numbers);
        assert_eq!(
            message.as_deref(),
            Some("Config created: ~/.config/chuch-term/config.toml")
        );

        let content = std::fs::read_to_string(&path).expect("config should exist");
        assert!(!content.contains("tab_width"));
        assert!(!content.contains("[theme]"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn reports_create_error_when_parent_is_not_directory() {
        let root = temp_path("error");
        std::fs::write(&root, "not a directory").expect("should create blocking file");
        let path = root.join("config.toml");

        let (_config, message) = load_config_from_path(&path);

        assert!(message
            .expect("create error should be reported")
            .starts_with("Config create error:"));

        let _ = std::fs::remove_file(root);
    }

    #[test]
    fn tolerates_legacy_keys() {
        let root = temp_path("legacy");
        std::fs::create_dir_all(&root).expect("temp dir");
        let path = root.join("config.toml");
        std::fs::write(
            &path,
            r##"
[editor]
line_numbers = false
relative_numbers = true
syntax_highlight = false
tab_width = 8

[theme]
accent = "#ffffff"

[clipboard]
strategy = "internal"
"##,
        )
        .expect("legacy config write");

        let (config, message) = load_config_from_path(&path);

        assert!(message.is_none());
        assert!(!config.editor.line_numbers);
        assert!(config.editor.relative_numbers);
        assert!(!config.editor.syntax_highlight);
        assert_eq!(config.clipboard.strategy, "internal");

        let _ = std::fs::remove_dir_all(root);
    }
}
