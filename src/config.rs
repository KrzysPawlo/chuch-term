use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const DISPLAY_CONFIG_PATH: &str = "~/.config/chuch-term/config.toml";

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

[render]
# "auto" = stable default, "rgb" = force 24-bit colours, "ansi256" = force 256-colour fallback
color_mode = "auto"

[theme]
# Hex colour strings — change any value and save; the editor picks it up within 2 seconds.
# Main accent colour: keybinding hints, selected items, highlights, active line number.
accent  = "#b0c4c8"
# Warning / confirmation colour: confirm-quit bar, command palette key column.
warning = "#ff9944"
# Dim text colour: descriptions, secondary UI text, inactive line numbers.
dim     = "#5a5a5a"
# Bottom bar background colour: status, hints, search, replace, go-to-line, save-as.
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
pub struct RenderSection {
    #[serde(default = "default_color_mode")]
    pub color_mode: String,
}

impl Default for RenderSection {
    fn default() -> Self {
        Self {
            color_mode: default_color_mode(),
        }
    }
}

fn default_color_mode() -> String { "auto".into() }

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
    /// Bottom bar background colour: status, hints, search, replace, go-to-line, save-as.
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
    pub render: RenderSection,
    #[serde(default)]
    pub theme: ThemeSection,
}

pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("chuch-term").join("config.toml"))
}

fn config_base_dir_from_env(
    xdg_config_home: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    xdg_config_home
        .map(PathBuf::from)
        .or_else(|| home.map(|h| PathBuf::from(h).join(".config")))
}

/// Returns the canonical config base directory without pulling in any external crate.
/// All supported platforms use `$XDG_CONFIG_HOME` or `~/.config`.
fn config_dir() -> Option<PathBuf> {
    config_base_dir_from_env(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

pub fn load_config() -> (EditorConfig, Option<String>) {
    let path = match config_path() {
        Some(path) => path,
        None => return (EditorConfig::default(), None),
    };
    load_config_from_path(&path)
}

pub fn load_existing_config() -> (Option<EditorConfig>, Option<String>) {
    let path = match config_path() {
        Some(path) => path,
        None => return (None, None),
    };

    if !path.exists() {
        return (None, None);
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<EditorConfig>(&content) {
            Ok(cfg) => {
                let (cfg, warn) = validate_config(cfg);
                (Some(cfg), warn)
            }
            Err(err) => (
                None,
                Some(format!("Config parse error: {err}")),
            ),
        },
        Err(err) => (
            None,
            Some(format!("Config read error: {err}")),
        ),
    }
}

pub fn load_config_from_path(path: &Path) -> (EditorConfig, Option<String>) {
    if !path.exists() {
        match create_default_config(path) {
            Ok(true) => {
                return (
                    EditorConfig::default(),
                    Some(format!("Config created: {DISPLAY_CONFIG_PATH}")),
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
    let mut warnings = Vec::new();
    const VALID_STRATEGIES: &[&str] = &["auto", "internal", "osc52"];
    if !VALID_STRATEGIES.contains(&cfg.clipboard.strategy.as_str()) {
        let bad = std::mem::replace(&mut cfg.clipboard.strategy, "auto".to_string());
        warnings.push(format!(
            "Config: unknown clipboard.strategy {bad:?}, using \"auto\""
        ));
    }
    const VALID_COLOR_MODES: &[&str] = &["auto", "rgb", "ansi256"];
    if !VALID_COLOR_MODES.contains(&cfg.render.color_mode.as_str()) {
        let bad = std::mem::replace(&mut cfg.render.color_mode, "auto".to_string());
        warnings.push(format!(
            "Config: unknown render.color_mode {bad:?}, using \"auto\""
        ));
    }
    // Clamp tab_width to a sensible range.
    cfg.editor.tab_width = cfg.editor.tab_width.clamp(1, 8);
    let warning = if warnings.is_empty() {
        None
    } else {
        Some(warnings.join("; "))
    };
    (cfg, warning)
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
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

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
        let expected = format!("Config created: {DISPLAY_CONFIG_PATH}");

        assert!(config.editor.line_numbers);
        assert_eq!(message.as_deref(), Some(expected.as_str()));

        let content = std::fs::read_to_string(&path).expect("config should exist");
        assert!(content.contains("tab_width"));    // valid field in DEFAULT_CONFIG_CONTENT
        assert!(content.contains("[render]"));
        assert!(content.contains("color_mode = \"auto\""));
        assert!(content.contains("[theme]"));      // theme section is now present
        assert!(content.contains("accent"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn create_message_uses_only_canonical_config_path() {
        let root = temp_path("create-message");
        let path = root.join("config.toml");

        let (_config, message) = load_config_from_path(&path);
        let message = message.expect("config create message");

        assert!(message.contains(DISPLAY_CONFIG_PATH));
        assert!(!message.contains("Application Support"));

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
        assert_eq!(config.render.color_mode, "auto");
        // Theme values from the legacy config are now loaded correctly.
        assert_eq!(config.theme.accent, "#ffffff");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn config_base_dir_prefers_xdg() {
        let dir = config_base_dir_from_env(
            Some(OsString::from("/tmp/xdg")),
            Some(OsString::from("/tmp/home")),
        )
        .expect("xdg dir");

        assert_eq!(dir, PathBuf::from("/tmp/xdg"));
    }

    #[test]
    fn config_base_dir_falls_back_to_dot_config() {
        let dir = config_base_dir_from_env(
            None,
            Some(OsString::from("/tmp/home")),
        )
        .expect("home dir");

        assert_eq!(dir, PathBuf::from("/tmp/home/.config"));
    }

    #[test]
    fn invalid_theme_hex_falls_back_to_defaults() {
        let mut theme = ThemeSection::default();
        theme.bg_bar = "not-a-color".to_string();
        theme.accent = "#12".to_string();

        assert_eq!(theme.bg_bar_rgb(), (18, 18, 18));
        assert_eq!(theme.accent_rgb(), (176, 196, 200));
    }

    #[test]
    fn invalid_render_mode_falls_back_to_auto() {
        let root = temp_path("render-mode");
        std::fs::create_dir_all(&root).expect("temp dir");
        let path = root.join("config.toml");
        std::fs::write(
            &path,
            r#"
[render]
color_mode = "neon"
"#,
        )
        .expect("render config write");

        let (config, warning) = load_config_from_path(&path);

        assert_eq!(config.render.color_mode, "auto");
        assert_eq!(
            warning.as_deref(),
            Some("Config: unknown render.color_mode \"neon\", using \"auto\"")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn load_existing_config_does_not_create_missing_file() {
        let _guard = env_lock().lock().expect("env test mutex");
        let root = temp_path("inspect");
        let previous_home = std::env::var_os("HOME");
        let previous_xdg = std::env::var_os("XDG_CONFIG_HOME");
        std::fs::create_dir_all(&root).expect("temp dir");
        std::env::set_var("HOME", &root);
        std::env::remove_var("XDG_CONFIG_HOME");

        let path = config_path().expect("config path");
        assert!(!path.exists());

        let (config, message) = load_existing_config();
        assert!(config.is_none());
        assert!(message.is_none());
        assert!(!path.exists());

        if let Some(home) = previous_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(xdg) = previous_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }

        let _ = std::fs::remove_dir_all(root);
    }
}
