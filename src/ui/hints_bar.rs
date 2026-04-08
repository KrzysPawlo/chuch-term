use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::{EditorMode, EditorState};

// ── Hardcoded (non-themed) design tokens ───────────────────────────────
const SEP_FG:   Color = Color::Rgb(45, 45, 45);    // #2d2d2d  separator ·
const WARN_SEP: Color = Color::Rgb(100, 60, 20);   // dim amber separator

/// One-row contextual hints bar rendered below the status bar.
pub struct HintsBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for HintsBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let y = area.top();

        // Resolve theme colours from config.
        let (r, g, b) = self.state.config.theme.bg_bar_rgb();
        let bg = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.accent_rgb();
        let accent = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.dim_rgb();
        let dim = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.warning_rgb();
        let warning = Color::Rgb(r, g, b);

        // Fill background — set both fg and bg explicitly to prevent style leaks
        // across ratatui frames (set_bg alone leaves fg from the previous frame).
        for x in area.left()..area.right() {
            buf[(x, y)].set_style(Style::default().bg(bg).fg(dim)).set_char(' ');
        }

        match self.state.mode {
            EditorMode::Normal => {
                if self.state.selection_anchor.is_some() {
                    render_selection_hints(area, buf, accent, dim, bg);
                } else {
                    render_normal(area, buf, self.state.previous_buffer.is_some(), accent, dim, bg);
                }
            }
            EditorMode::ConfirmQuit => render_confirm(area, buf, warning, bg),
            EditorMode::Help => render_help(area, buf, dim, bg),
            EditorMode::Search | EditorMode::GoToLine => {
                // These modes render their own bar widget (search_bar / goto_bar).
            }
            EditorMode::Replace | EditorMode::CommandPalette
            | EditorMode::SaveAs | EditorMode::Settings => {
                // Respective overlays/bars handle these modes.
            }
        }
    }
}

// ── Render helpers ─────────────────────────────────────────────────────

/// Write `text` at (x, y) with `style`, clipped to `max_x`. Returns new x.
fn put(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style, max_x: u16) -> u16 {
    let mut cx = x;
    for ch in text.chars() {
        if cx >= max_x {
            break;
        }
        buf[(cx, y)].set_char(ch).set_style(style);
        cx += 1;
    }
    cx
}

/// Render a sequence of (key, description) hint pairs separated by ` · `.
fn render_hints(
    area: Rect,
    buf: &mut Buffer,
    hints: &[(&str, &str)],
    key_style: Style,
    desc_style: Style,
    sep_style: Style,
) {
    let y = area.top();
    let max_x = area.right();
    let mut x = area.left() + 1; // 1-cell left padding

    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            x = put(buf, x, y, "  \u{00b7}  ", sep_style, max_x);
        }
        x = put(buf, x, y, key, key_style, max_x);
        x = put(buf, x, y, "  ", desc_style, max_x);
        x = put(buf, x, y, desc, desc_style, max_x);
        let _ = x;
    }
}

fn render_normal(area: Rect, buf: &mut Buffer, has_prev: bool, accent: Color, dim: Color, bg: Color) {
    let key_style  = Style::default().fg(accent).bg(bg).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(dim).bg(bg);
    let sep_style  = Style::default().fg(SEP_FG).bg(bg);

    if has_prev {
        render_hints(
            area, buf,
            &[
                ("^S", "Save"),
                ("^Z", "Undo"),
                ("^F", "Find"),
                ("^P", "Commands"),
                ("^O", "Back"),
                ("^H", "Help"),
            ],
            key_style, desc_style, sep_style,
        );
    } else {
        render_hints(
            area, buf,
            &[
                ("^S", "Save"),
                ("^Z", "Undo"),
                ("^F", "Find"),
                ("^P", "Commands"),
                ("^H", "Help"),
            ],
            key_style, desc_style, sep_style,
        );
    }
}

fn render_selection_hints(area: Rect, buf: &mut Buffer, accent: Color, dim: Color, bg: Color) {
    let key_style  = Style::default().fg(accent).bg(bg).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(dim).bg(bg);
    let sep_style  = Style::default().fg(SEP_FG).bg(bg);

    render_hints(
        area, buf,
        &[
            ("^C", "Copy"),
            ("^X", "Cut"),
            ("^V", "Paste"),
            ("Esc", "Clear"),
        ],
        key_style, desc_style, sep_style,
    );
}

fn render_confirm(area: Rect, buf: &mut Buffer, warning: Color, bg: Color) {
    let key_style  = Style::default().fg(warning).bg(bg).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(warning).bg(bg);
    let sep_style  = Style::default().fg(WARN_SEP).bg(bg);

    render_hints(
        area, buf,
        &[
            ("^Q", "Force Quit"),
            ("^S", "Save & Quit"),
            ("Esc", "Cancel"),
        ],
        key_style, desc_style, sep_style,
    );
}

fn render_help(area: Rect, buf: &mut Buffer, dim: Color, bg: Color) {
    let text = "Esc  Close Help";
    let text_len = text.chars().count() as u16;
    let x = area
        .left()
        .saturating_add(area.width.saturating_sub(text_len) / 2);
    let style = Style::default().fg(dim).bg(bg);
    put(buf, x, area.top(), text, style, area.right());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::EditorState;

    #[test]
    fn hints_bar_uses_theme_bg_bar() {
        let mut state = EditorState::new_empty();
        state.config.theme.bg_bar = "#224466".to_string();

        let area = Rect::new(0, 0, 16, 1);
        let mut buf = Buffer::empty(area);
        HintsBar { state: &state }.render(area, &mut buf);

        for x in area.left()..area.right() {
            assert_eq!(buf[(x, 0)].bg, Color::Rgb(34, 68, 102));
        }
    }
}
