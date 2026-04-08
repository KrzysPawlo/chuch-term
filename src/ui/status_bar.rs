use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::editor::{EditorMode, EditorState};
use crate::syntax::Language;

// Hardcoded (non-themed) design tokens.
const TEXT_MAIN: Color = Color::Rgb(220, 220, 220);
const LANG_FG:   Color = Color::Rgb(130, 170, 150);  // muted teal — language label

pub struct StatusBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        // Resolve theme colours.
        let (r, g, b) = self.state.config.theme.bg_bar_rgb();
        let bg = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.dim_rgb();
        let dim = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.warning_rgb();
        let warning = Color::Rgb(r, g, b);

        for x in area.left()..area.right() {
            buf[(x, area.top())].set_style(Style::default().bg(bg).fg(dim)).set_char(' ');
        }

        match self.state.mode {
            EditorMode::ConfirmQuit => render_confirm_quit(area, buf, warning, bg),
            _ => render_normal(self.state, area, buf, bg, dim),
        }
    }
}

fn language_name(lang: Language) -> Option<&'static str> {
    match lang {
        Language::Rust => Some("Rust"),
        Language::Python => Some("Python"),
        Language::JavaScript => Some("JavaScript"),
        Language::Go => Some("Go"),
        Language::Toml => Some("TOML"),
        Language::Yaml => Some("YAML"),
        Language::Shell => Some("Shell"),
        Language::Markdown => Some("Markdown"),
        Language::Proto => Some("Proto3"),
        Language::Plain => None,
    }
}

fn render_normal(state: &EditorState, area: Rect, buf: &mut Buffer, bg: Color, dim: Color) {
    let y = area.top();
    let width = area.width;
    if width == 0 {
        return;
    }

    let (r, g, b) = state.config.theme.accent_rgb();
    let accent = Color::Rgb(r, g, b);

    let left_text = format!(
        " {}{}",
        state.file_display_name(),
        if state.buffer.dirty { " [+]" } else { "" }
    );

    let row_num = (state.cursor.row + 1).to_string();
    let col_num = (state.cursor.col + 1).to_string();

    let right_width: usize;
    let right_x: u16;

    if let Some(message) = &state.status_message {
        // Status message overrides position display.
        let msg = truncate_to_width(message, width as usize);
        right_width = msg.width();
        right_x = area.right().saturating_sub(right_width as u16);
        let _ = write_text(buf, right_x, y, &msg, Style::default().fg(dim).bg(bg), area.right());
        let left_budget = width.saturating_sub(right_width as u16 + 1);
        render_left(buf, &left_text, y, area.left(), left_budget, accent, bg);
        return;
    }

    // Normal state: language + styled position.
    let lang_prefix = language_name(state.language())
        .map(|l| format!("{l}   "))
        .unwrap_or_default();
    let right_full = format!("{lang_prefix}Ln {row_num}  Col {col_num} ");
    right_width = right_full.width();
    right_x = area.right().saturating_sub(right_width as u16);

    let mut x = right_x;
    if !lang_prefix.is_empty() {
        x = write_text(buf, x, y, &lang_prefix, Style::default().fg(LANG_FG).bg(bg), area.right());
    }
    x = write_text(buf, x, y, "Ln ",     Style::default().fg(dim).bg(bg), area.right());
    x = write_text(buf, x, y, &row_num,  Style::default().fg(accent).bg(bg).add_modifier(Modifier::BOLD), area.right());
    x = write_text(buf, x, y, "  Col ",  Style::default().fg(dim).bg(bg), area.right());
    x = write_text(buf, x, y, &col_num,  Style::default().fg(accent).bg(bg).add_modifier(Modifier::BOLD), area.right());
    let _ = write_text(buf, x, y, " ", Style::default().fg(dim).bg(bg), area.right());

    let left_budget = width.saturating_sub(right_width as u16 + 1);
    render_left(buf, &left_text, y, area.left(), left_budget, accent, bg);
}

fn render_left(buf: &mut Buffer, text: &str, y: u16, start_x: u16, budget: u16, accent: Color, bg: Color) {
    let display = truncate_to_width(text, budget as usize);
    let has_dirty = display.contains("[+]");
    let mut x = start_x;
    for ch in display.chars() {
        let style = if has_dirty && matches!(ch, '[' | '+' | ']') {
            Style::default().fg(accent).bg(bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT_MAIN).bg(bg)
        };
        buf[(x, y)].set_char(ch).set_style(style);
        x += UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
    }
}

fn render_confirm_quit(area: Rect, buf: &mut Buffer, warning: Color, bg: Color) {
    let message =
        " Unsaved changes. Ctrl+Q force quit  ·  Ctrl+S save & quit  ·  Esc cancel ";
    let _ = write_text(
        buf,
        area.left(),
        area.top(),
        &truncate_to_width(message, area.width as usize),
        Style::default().fg(warning).bg(bg).add_modifier(Modifier::BOLD),
        area.right(),
    );
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if text.width() <= max_width {
        return text.to_string();
    }
    if max_width == 1 {
        return "…".to_string();
    }

    let mut out = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let width = UnicodeWidthChar::width(ch).unwrap_or(1);
        if used + width + 1 > max_width {
            break;
        }
        out.push(ch);
        used += width;
    }
    out.push('…');
    out
}

fn write_text(
    buf: &mut Buffer,
    mut x: u16,
    y: u16,
    text: &str,
    style: Style,
    max_x: u16,
) -> u16 {
    for ch in text.chars() {
        if x >= max_x {
            break;
        }
        buf[(x, y)].set_char(ch).set_style(style);
        let width = UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
        if width == 2 && x + 1 < max_x {
            buf[(x + 1, y)].set_char(' ').set_style(style);
        }
        x += width.max(1);
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::EditorState;
    use ratatui::layout::Rect;

    #[test]
    fn truncation_is_utf8_safe() {
        let truncated = truncate_to_width(" żółw-file.txt [+]", 8);
        assert!(truncated.ends_with('…'));
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn truncation_handles_tiny_width() {
        assert_eq!(truncate_to_width("abc", 0), "");
        assert_eq!(truncate_to_width("abc", 1), "…");
    }

    #[test]
    fn status_bar_uses_theme_bg_bar() {
        let mut state = EditorState::new_empty();
        state.status_message = None;
        state.config.theme.bg_bar = "#112233".to_string();

        let area = Rect::new(0, 0, 12, 1);
        let mut buf = Buffer::empty(area);
        StatusBar { state: &state }.render(area, &mut buf);

        for x in area.left()..area.right() {
            assert_eq!(buf[(x, 0)].bg, Color::Rgb(17, 34, 51));
        }
    }
}
