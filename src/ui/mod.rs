pub mod command_palette;
pub mod editor_view;
pub mod goto_bar;
pub mod help_overlay;
pub mod hints_bar;
pub mod line_numbers;
pub mod replace_bar;
pub mod saveas_bar;
pub mod search_bar;
pub mod settings_overlay;
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
use saveas_bar::SaveAsBar;
use search_bar::SearchBar;
use settings_overlay::SettingsOverlay;
use status_bar::StatusBar;

/// Draw the full UI into the ratatui frame.
/// Takes `&mut EditorState` so it can record the editor area bounds for mouse hit-testing.
pub fn draw(frame: &mut Frame, state: &mut EditorState) {
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

    // Store editor area bounds for mouse click translation.
    state.editor_area_left = editor_area.left();
    state.editor_area_top = editor_area.top();
    state.editor_area_right = editor_area.right();
    state.editor_area_bottom = editor_area.bottom();

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
        EditorMode::SaveAs => {
            frame.render_widget(SaveAsBar { state }, hints_area);
        }
        EditorMode::CommandPalette | EditorMode::Settings => {
            // Overlays cover the hints bar — nothing rendered here.
        }
        _ => {
            frame.render_widget(HintsBar { state }, hints_area);
        }
    }

    // ── Overlays (cover everything) ────────────────────────────────────
    if state.mode == EditorMode::Help {
        frame.render_widget(HelpOverlay { state }, area);
        return;
    }

    if state.mode == EditorMode::CommandPalette {
        frame.render_widget(CommandPalette { state }, area);
        return;
    }

    if state.mode == EditorMode::Settings {
        frame.render_widget(SettingsOverlay { state }, area);
        return;
    }

    // ── Terminal cursor position ───────────────────────────────────────
    let viewport_height = editor_area.height as usize;
    let cursor_screen_row = state.cursor.row.saturating_sub(state.viewport.offset_row);

    if cursor_screen_row < viewport_height {
        let display_col = EditorView::cursor_display_col(state);
        let screen_x = editor_area.left() + display_col.min(editor_area.width.saturating_sub(1));
        let screen_y = editor_area.top() + cursor_screen_row as u16;
        frame.set_cursor_position((screen_x, screen_y));
    }
}
