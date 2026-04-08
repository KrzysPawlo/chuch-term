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

    // ── Mouse / layout ───────────────────────────────────────────────────
    /// Left edge of the editor area in terminal columns (updated every frame).
    pub editor_area_left: u16,
    /// Top edge of the editor area in terminal rows (updated every frame).
    pub editor_area_top: u16,

    // ── Command palette ──────────────────────────────────────────────────
    pub palette_query: String,
    pub palette_matches: Vec<usize>,
    pub palette_cursor: usize,

    // ── Config ───────────────────────────────────────────────────────────
    pub config: crate::config::EditorConfig,
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

    pub fn new_empty() -> Self {
        let (config, config_msg) = crate::config::load_config();
        let line_number_mode = Self::line_number_mode_for(&config);
        let config_mtime = crate::config::config_mtime();
        Self {
            buffer: TextBuffer::new_empty(),
            cursor: Cursor::new(),
            viewport: Viewport::new(),
            viewport_height: 0,
            mode: EditorMode::Normal,
            status_message: config_msg,
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
            editor_area_left: 0,
            editor_area_top: 0,
            palette_query: String::new(),
            palette_matches: (0..crate::commands::COMMANDS.len()).collect(),
            palette_cursor: 0,
            config,
            config_mtime,
            previous_buffer: None,
        }
    }

    /// Open a new empty buffer pre-associated with `path` (file does not need to exist yet).
    pub fn new_with_path(path: &Path) -> Self {
        let (config, config_msg) = crate::config::load_config();
        let line_number_mode = Self::line_number_mode_for(&config);
        let config_mtime = crate::config::config_mtime();
        let mut buffer = TextBuffer::new_empty();
        buffer.file_path = Some(path.to_path_buf());
        Self {
            buffer,
            cursor: Cursor::new(),
            viewport: Viewport::new(),
            viewport_height: 0,
            mode: EditorMode::Normal,
            status_message: config_msg,
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
            editor_area_left: 0,
            editor_area_top: 0,
            palette_query: String::new(),
            palette_matches: (0..crate::commands::COMMANDS.len()).collect(),
            palette_cursor: 0,
            config,
            config_mtime,
            previous_buffer: None,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let (config, config_msg) = crate::config::load_config();
        let line_number_mode = Self::line_number_mode_for(&config);
        let config_mtime = crate::config::config_mtime();
        let buffer = TextBuffer::from_file(path)?;
        Ok(Self {
            buffer,
            cursor: Cursor::new(),
            viewport: Viewport::new(),
            viewport_height: 0,
            mode: EditorMode::Normal,
            status_message: config_msg,
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
            editor_area_left: 0,
            editor_area_top: 0,
            palette_query: String::new(),
            palette_matches: (0..crate::commands::COMMANDS.len()).collect(),
            palette_cursor: 0,
            config,
            config_mtime,
            previous_buffer: None,
        })
    }

    pub fn apply_config(&mut self, config: crate::config::EditorConfig) {
        self.line_number_mode = Self::line_number_mode_for(&config);
        self.config = config;
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
        let cur = self.cursor;
        let (a_row, a_col) = (anchor.row, anchor.col);
        let (c_row, c_col) = (cur.row, cur.col);
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

        state.apply_config(config.clone());

        assert_eq!(state.line_number_mode, LineNumberMode::Relative);
        assert!(!state.config.editor.syntax_highlight);
        assert_eq!(state.config.clipboard.strategy, "internal");

        config.editor.line_numbers = false;
        state.apply_config(config);
        assert_eq!(state.line_number_mode, LineNumberMode::Off);
    }
}
