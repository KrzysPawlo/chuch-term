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
auto_indent = true
expand_tabs = true
tab_width = 4
indent_guides = false
indent_errors = false
# indent_error_bg = [70, 20, 20]  # RGB colour of the error background (default)

[clipboard]
# "auto" = detect system clipboard, "internal" = never use system clipboard, "osc52" = force OSC-52
strategy = "auto"

[theme]
# Hex colour strings — change any value and save; the editor picks it up within 2 seconds.
# Main accent colour: keybinding hints, selected items, highlights, active line number.
accent  = "#b0c4c8"
# Warning / confirmation colour: confirm-quit bar, command palette key column.
warning = "#ff9944"
# Dim text colour: descriptions, secondary UI text, inactive line numbers.
dim     = "#5a5a5a"
# Status and hints bar background colour.
bg_bar  = "#121212"
"##;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSection {
    pub line_numbers: bool,
    pub relative_numbers: bool,
    pub syntax_highlight: bool,

    // Indentation behaviour
    #[serde(default = "default_true")]
    pub auto_indent: bool,
    #[serde(default = "default_true")]
    pub expand_tabs: bool,
    #[serde(default = "default_tab_width")]
    pub tab_width: u8,

    // Visual guides
    #[serde(default)]
    pub indent_guides: bool,
    #[serde(default)]
    pub indent_errors: bool,
    #[serde(default = "default_indent_error_bg")]
    pub indent_error_bg: [u8; 3],
}

impl Default for EditorSection {
    fn default() -> Self {
        Self {
            line_numbers: true,
            relative_numbers: false,
            syntax_highlight: true,
            auto_indent: true,
            expand_tabs: true,
            tab_width: 4,
            indent_guides: false,
            indent_errors: false,
            indent_error_bg: [70, 20, 20],
        }
    }
}

fn default_true() -> bool { true }
fn default_tab_width() -> u8 { 4 }
fn default_indent_error_bg() -> [u8; 3] { [70, 20, 20] }

// ── Theme ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSection {
    /// Main accent colour: keybinding hints, selected items, highlights.
    #[serde(default = "default_accent")]
    pub accent: String,
    /// Warning / confirmation colour: confirm-quit bar, command palette key column.
    #[serde(default = "default_warning")]
    pub warning: String,
    /// Dim text colour: descriptions and secondary UI elements.
    #[serde(default = "default_dim")]
    pub dim: String,
    /// Status and hints bar background colour.
    #[serde(default = "default_bg_bar")]
    pub bg_bar: String,
}

fn default_accent()  -> String { "#b0c4c8".into() }
fn default_warning() -> String { "#ff9944".into() }
fn default_dim()     -> String { "#5a5a5a".into() }
fn default_bg_bar()  -> String { "#121212".into() }

impl Default for ThemeSection {
    fn default() -> Self {
        Self {
            accent:  default_accent(),
            warning: default_warning(),
            dim:     default_dim(),
            bg_bar:  default_bg_bar(),
        }
    }
}

impl ThemeSection {
    pub fn accent_rgb(&self)  -> (u8, u8, u8) { parse_hex_rgb(&self.accent,  176, 196, 200) }
    pub fn warning_rgb(&self) -> (u8, u8, u8) { parse_hex_rgb(&self.warning, 255, 153,  68) }
    pub fn dim_rgb(&self)     -> (u8, u8, u8) { parse_hex_rgb(&self.dim,      90,  90,  90) }
    pub fn bg_bar_rgb(&self)  -> (u8, u8, u8) { parse_hex_rgb(&self.bg_bar,   18,  18,  18) }
}

/// Parse a hex colour string (`"#rrggbb"` or `"rrggbb"`) to an RGB tuple.
/// Falls back to the provided defaults when the string is malformed.
fn parse_hex_rgb(s: &str, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        if let (Ok(rv), Ok(gv), Ok(bv)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return (rv, gv, bv);
        }
    }
    (r, g, b)
}

// ── Clipboard ─────────────────────────────────────────────────────────

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
    #[serde(default)]
    pub theme: ThemeSection,
}

pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("chuch-term").join("config.toml"))
}

/// Returns the OS config base directory without pulling in any external crate.
/// Matches the behaviour of `dirs::config_dir()`:
///   macOS  → ~/Library/Application Support
///   Linux  → $XDG_CONFIG_HOME or ~/.config
fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    return std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Library").join("Application Support"));

    #[cfg(not(target_os = "macos"))]
    return std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")));
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

/// Persist the current in-memory config back to disk (used by the Settings overlay).
/// Overwrites the file with clean TOML — comments from the original file are not preserved.
pub fn save_config(config: &EditorConfig) -> anyhow::Result<()> {
    use anyhow::Context as _;
    let path = config_path()
        .context("Cannot determine config path")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create config dir: {}", parent.display()))?;
    }
    let content = toml::to_string_pretty(config)
        .context("Cannot serialise config")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Cannot write config: {}", path.display()))?;
    Ok(())
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
    // Clamp tab_width to a sensible range.
    cfg.editor.tab_width = cfg.editor.tab_width.clamp(1, 8);
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
        assert!(content.contains("tab_width"));    // valid field in DEFAULT_CONFIG_CONTENT
        assert!(content.contains("[theme]"));      // theme section is now present
        assert!(content.contains("accent"));

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
        // Theme values from the legacy config are now loaded correctly.
        assert_eq!(config.theme.accent, "#ffffff");

        let _ = std::fs::remove_dir_all(root);
    }
}
