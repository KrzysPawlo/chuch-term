pub mod buffer;
pub mod cursor;
pub mod history;
pub mod search;
pub mod viewport;

pub use buffer::TextBuffer;
pub use cursor::Cursor;
pub use search::SearchMatch;
pub use viewport::Viewport;

use std::path::Path;
use anyhow::Result;

/// Editor interaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Normal editing.
    Normal,
    /// User pressed Ctrl+Q with unsaved changes — waiting for confirmation.
    ConfirmQuit,
    /// Full-screen help overlay is visible.
    Help,
    /// Incremental search mode.
    Search,
    /// Go-to-line input mode.
    GoToLine,
    /// Command palette overlay.
    CommandPalette,
    /// Find and replace mode.
    Replace,
    /// Save-as prompt — user types a filename to save the buffer to.
    SaveAs,
    /// Interactive settings panel.
    Settings,
    /// Dedicated shortcut customization overlay.
    Keybindings,
    /// Dedicated command-alias editor overlay.
    CommandAlias,
}

/// Line number display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineNumberMode {
    Off,
    Absolute,
    Relative,
}

/// Top-level editor state: buffer + cursor + viewport + mode.
pub struct EditorState {
    pub buffer: TextBuffer,
    pub cursor: Cursor,
    pub viewport: Viewport,
    /// Actual visible rows in the terminal (updated each frame before drawing).
    pub viewport_height: usize,
    pub mode: EditorMode,
    /// Transient status message shown in the status bar (errors, info).
    pub status_message: Option<String>,
    /// Set to true when the app should exit.
    pub should_quit: bool,
    /// Mode to restore when closing Help (Normal or ConfirmQuit).
    pub pre_help_mode: EditorMode,

    // ── Undo / Redo ──────────────────────────────────────────────────────
    pub history: crate::editor::history::History,

    // ── Search ───────────────────────────────────────────────────────────
    pub search_query: String,
    pub search_results: Vec<SearchMatch>,
    pub search_result_idx: usize,
    pub replace_query: String,
    pub search_case_sensitive: bool,

    // ── Selection ────────────────────────────────────────────────────────
    pub selection_anchor: Option<Cursor>,
    pub clipboard: String,

    // ── Line numbers ─────────────────────────────────────────────────────
    pub line_number_mode: LineNumberMode,

    // ── Go-to-line ───────────────────────────────────────────────────────
    pub goto_input: String,

    // ── Save-as ──────────────────────────────────────────────────────────
    pub saveas_input: String,

    // ── Settings overlay ─────────────────────────────────────────────────
    /// Index of the currently selected row in the Settings overlay.
    pub settings_cursor: usize,
    /// Index of the currently selected shortcut in the keybindings overlay.
    pub keybindings_cursor: usize,
    /// True while waiting for a new key token in the keybindings overlay.
    pub keybinding_capture: bool,
    /// Draft text while editing the optional managed command alias.
    pub command_alias_input: String,

    // ── Mouse / layout ───────────────────────────────────────────────────
    /// Left edge of the editor area in terminal columns (updated every frame).
    pub editor_area_left: u16,
    /// Top edge of the editor area in terminal rows (updated every frame).
    pub editor_area_top: u16,
    /// Right edge of the editor area in terminal columns (updated every frame).
    pub editor_area_right: u16,
    /// Bottom edge of the editor area in terminal rows (updated every frame).
    pub editor_area_bottom: u16,

    // ── Command palette ──────────────────────────────────────────────────
    pub palette_query: String,
    pub palette_matches: Vec<usize>,
    pub palette_cursor: usize,

    // ── Config ───────────────────────────────────────────────────────────
    pub config: crate::config::EditorConfig,
    pub active_shortcuts: crate::shortcuts::ActiveShortcuts,
    pub render_decision: crate::color::RenderDecision,
    pub palette: crate::color::Palette,
    pub config_mtime: Option<std::time::SystemTime>,

    // ── Previous buffer (for GoBackBuffer after OpenConfig) ───────────────
    pub previous_buffer: Option<(TextBuffer, Cursor)>,
}

impl EditorState {
    pub(crate) fn line_number_mode_for(config: &crate::config::EditorConfig) -> LineNumberMode {
        if !config.editor.line_numbers {
            LineNumberMode::Off
        } else if config.editor.relative_numbers {
            LineNumberMode::Relative
        } else {
            LineNumberMode::Absolute
        }
    }

    fn resolve_rendering(
        config: &crate::config::EditorConfig,
    ) -> (crate::color::RenderDecision, crate::color::Palette) {
        let env = crate::color::TerminalEnv::detect();
        let render_decision = crate::color::resolve_render_decision(config, &env);
        let palette = crate::color::build_palette(config, render_decision.effective);
        (render_decision, palette)
    }

    fn build(buffer: TextBuffer, config: crate::config::EditorConfig, status_message: Option<String>) -> Self {
        let line_number_mode = Self::line_number_mode_for(&config);
        let config_mtime = crate::config::config_mtime();
        let (render_decision, palette) = Self::resolve_rendering(&config);
        let active_shortcuts = crate::shortcuts::ActiveShortcuts::resolve(&config.shortcuts)
            .expect("validated config should resolve active shortcuts");

        Self {
            buffer,
            cursor: Cursor::new(),
            viewport: Viewport::new(),
            viewport_height: 0,
            mode: EditorMode::Normal,
            status_message,
            should_quit: false,
            pre_help_mode: EditorMode::Normal,
            history: crate::editor::history::History::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_idx: 0,
            replace_query: String::new(),
            search_case_sensitive: false,
            selection_anchor: None,
            clipboard: String::new(),
            line_number_mode,
            goto_input: String::new(),
            saveas_input: String::new(),
            settings_cursor: 0,
            keybindings_cursor: 0,
            keybinding_capture: false,
            command_alias_input: String::new(),
            editor_area_left: 0,
            editor_area_top: 0,
            editor_area_right: 0,
            editor_area_bottom: 0,
            palette_query: String::new(),
            palette_matches: (0..crate::commands::COMMANDS.len()).collect(),
            palette_cursor: 0,
            config,
            active_shortcuts,
            render_decision,
            palette,
            config_mtime,
            previous_buffer: None,
        }
    }

    pub fn new_empty() -> Self {
        let (config, config_msg) = crate::config::load_config();
        Self::build(TextBuffer::new_empty(), config, config_msg)
    }

    /// Open a new empty buffer pre-associated with `path` (file does not need to exist yet).
    pub fn new_with_path(path: &Path) -> Self {
        let (config, config_msg) = crate::config::load_config();
        let mut buffer = TextBuffer::new_empty();
        buffer.file_path = Some(path.to_path_buf());
        Self::build(buffer, config, config_msg)
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let (config, config_msg) = crate::config::load_config();
        let buffer = TextBuffer::from_file(path)?;
        Ok(Self::build(buffer, config, config_msg))
    }

    pub fn apply_config(&mut self, config: crate::config::EditorConfig) {
        let (render_decision, palette) = Self::resolve_rendering(&config);
        let active_shortcuts = crate::shortcuts::ActiveShortcuts::resolve(&config.shortcuts)
            .expect("validated config should resolve active shortcuts");
        self.line_number_mode = Self::line_number_mode_for(&config);
        self.config = config;
        self.active_shortcuts = active_shortcuts;
        self.render_decision = render_decision;
        self.palette = palette;
    }

    pub fn selected_keybinding_action(&self) -> Option<crate::shortcuts::ShortcutAction> {
        crate::shortcuts::configurable_actions()
            .get(self.keybindings_cursor)
            .copied()
    }

    /// Display name for the status bar (filename or "[New File]").
    pub fn file_display_name(&self) -> String {
        self.buffer.display_name()
    }

    /// True when the buffer is a brand-new empty file (welcome screen condition).
    pub fn is_welcome_state(&self) -> bool {
        self.buffer.file_path.is_none()
            && self.buffer.line_count() == 1
            && self.buffer.line(0).is_empty()
    }

    /// Get the sorted selection range if one exists.
    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.selection_anchor?;
        let (a_row, a_col) = self.buffer.clamp_position(anchor.row, anchor.col);
        let (c_row, c_col) = self.buffer.clamp_position(self.cursor.row, self.cursor.col);
        if (a_row, a_col) <= (c_row, c_col) {
            Some(((a_row, a_col), (c_row, c_col)))
        } else {
            Some(((c_row, c_col), (a_row, a_col)))
        }
    }

    /// Detect the language from the file extension.
    pub fn language(&self) -> crate::syntax::Language {
        crate::syntax::detect_language(self.buffer.file_path.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_config_updates_live_editor_state() {
        let mut state = EditorState::new_empty();
        assert_eq!(state.line_number_mode, LineNumberMode::Absolute);
        assert!(state.config.editor.syntax_highlight);

        let mut config = crate::config::EditorConfig::default();
        config.editor.line_numbers = true;
        config.editor.relative_numbers = true;
        config.editor.syntax_highlight = false;
        config.clipboard.strategy = "internal".to_string();
        config.render.color_mode = "ansi256".to_string();

        state.apply_config(config.clone());

        assert_eq!(state.line_number_mode, LineNumberMode::Relative);
        assert!(!state.config.editor.syntax_highlight);
        assert_eq!(state.config.clipboard.strategy, "internal");
        assert_eq!(state.render_decision.effective.as_str(), "ansi256");

        config.editor.line_numbers = false;
        state.apply_config(config);
        assert_eq!(state.line_number_mode, LineNumberMode::Off);
    }

    #[test]
    fn selection_range_clamps_utf8_boundaries() {
        let mut state = EditorState::new_empty();
        state.buffer.lines = vec!["zażółć".to_string()];
        state.selection_anchor = Some(Cursor { row: 0, col: 3 });
        state.cursor = Cursor { row: 0, col: 8 };

        assert_eq!(state.selection_range(), Some(((0, 2), (0, 8))));
    }
}
