pub mod command_palette;
pub mod editor_view;
pub mod goto_bar;
pub mod help_overlay;
pub mod hints_bar;
pub mod line_numbers;
pub mod replace_bar;
pub mod search_bar;
pub mod status_bar;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};
use crate::editor::{EditorMode, EditorState, LineNumberMode};
use command_palette::CommandPalette;
use editor_view::EditorView;
use goto_bar::GotoBar;
use help_overlay::HelpOverlay;
use hints_bar::HintsBar;
use line_numbers::{gutter_width, LineNumbersGutter};
use replace_bar::ReplaceBar;
use search_bar::SearchBar;
use status_bar::StatusBar;

/// Draw the full UI into the ratatui frame.
pub fn draw(frame: &mut Frame, state: &EditorState) {
    let area = frame.area();

    // Split: editor area + status bar (1 row) + hints bar (1 row).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),     // editor + gutter
            Constraint::Length(1),  // status bar
            Constraint::Length(1),  // hints bar
        ])
        .split(area);

    let editor_chunk = chunks[0];
    let status_area = chunks[1];
    let hints_area = chunks[2];

    // Split editor area horizontally for line numbers gutter if needed.
    let (gutter_area, editor_area) = if state.line_number_mode != LineNumberMode::Off {
        let gw = gutter_width(state.buffer.line_count());
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(gw), Constraint::Min(1)])
            .split(editor_chunk);
        (Some(h_chunks[0]), h_chunks[1])
    } else {
        (None, editor_chunk)
    };

    // ── Render base layers ─────────────────────────────────────────────
    if let Some(g_area) = gutter_area {
        frame.render_widget(LineNumbersGutter { state }, g_area);
    }
    frame.render_widget(EditorView { state }, editor_area);
    frame.render_widget(StatusBar { state }, status_area);

    // ── Render hints bar based on mode ─────────────────────────────────
    match state.mode {
        EditorMode::Search => {
            frame.render_widget(SearchBar { state }, hints_area);
        }
        EditorMode::Replace => {
            frame.render_widget(ReplaceBar { state }, hints_area);
        }
        EditorMode::GoToLine => {
            frame.render_widget(GotoBar { state }, hints_area);
        }
        EditorMode::CommandPalette => {
            // No hints bar content — palette covers everything
        }
        _ => {
            frame.render_widget(HintsBar { state }, hints_area);
        }
    }

    // ── Overlays (cover everything) ────────────────────────────────────
    if state.mode == EditorMode::Help {
        frame.render_widget(HelpOverlay, area);
        return;
    }

    if state.mode == EditorMode::CommandPalette {
        frame.render_widget(CommandPalette { state }, area);
        return;
    }

    // ── Terminal cursor position ───────────────────────────────────────
    let viewport_height = editor_area.height as usize;
    let cursor_screen_row = state.cursor.row.saturating_sub(state.viewport.offset_row);

    // Don't show cursor in palette or other overlay modes
    if matches!(state.mode, EditorMode::CommandPalette | EditorMode::Help) {
        return;
    }

    if cursor_screen_row < viewport_height {
        let display_col = EditorView::cursor_display_col(state);
        let screen_x = editor_area.left() + display_col.min(editor_area.width.saturating_sub(1));
        let screen_y = editor_area.top() + cursor_screen_row as u16;
        frame.set_cursor_position((screen_x, screen_y));
    }
}
