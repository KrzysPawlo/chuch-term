use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::editor::{EditorMode, EditorState};
use crate::syntax::Language;

const BG: Color = Color::Rgb(26, 26, 26);
const TEXT_DIM: Color = Color::Rgb(136, 136, 136);
const TEXT_MAIN: Color = Color::Rgb(220, 220, 220);
const ACCENT: Color = Color::Rgb(176, 196, 200);
const WARN: Color = Color::Rgb(255, 153, 68);
const LANG_FG: Color = Color::Rgb(130, 170, 150);

pub struct StatusBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        for x in area.left()..area.right() {
            buf[(x, area.top())].set_bg(BG).set_char(' ');
        }

        match self.state.mode {
            EditorMode::ConfirmQuit => render_confirm_quit(area, buf),
            _ => render_normal(self.state, area, buf),
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

fn render_normal(state: &EditorState, area: Rect, buf: &mut Buffer) {
    let y = area.top();
    let width = area.width;
    if width == 0 {
        return;
    }

    let left_text = format!(
        " {}{}",
        state.file_display_name(),
        if state.buffer.dirty { " [+]" } else { "" }
    );
    let position = format!("{}:{}", state.cursor.row + 1, state.cursor.col + 1);
    let right_text = if let Some(message) = &state.status_message {
        message.clone()
    } else if let Some(language) = language_name(state.language()) {
        format!("{language}  {position}")
    } else {
        position.clone()
    };

    let right_reserved = right_text.width().min(width as usize);
    let left_budget = width.saturating_sub(right_reserved as u16 + 1);
    let left_display = truncate_to_width(&left_text, left_budget as usize);
    let left_has_dirty_marker = left_display.contains("[+]");

    let mut left_x = area.left();
    for ch in left_display.chars() {
        if left_x >= area.right() {
            break;
        }
        let style = if left_has_dirty_marker && matches!(ch, '[' | '+' | ']') {
            Style::default().fg(ACCENT).bg(BG).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT_MAIN).bg(BG)
        };
        buf[(left_x, y)].set_char(ch).set_style(style);
        left_x += UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
    }

    let right_x = area
        .right()
        .saturating_sub(truncate_to_width(&right_text, width as usize).width() as u16);

    if state.status_message.is_none() {
        if let Some(language) = language_name(state.language()) {
            let language_prefix = format!("{language}  ");
            let x = write_text(
                buf,
                right_x,
                y,
                &language_prefix,
                Style::default().fg(LANG_FG).bg(BG),
                area.right(),
            );
            let _ = write_text(
                buf,
                x,
                y,
                &position,
                Style::default().fg(TEXT_DIM).bg(BG),
                area.right(),
            );
            return;
        }
    }

    let _ = write_text(
        buf,
        right_x,
        y,
        &truncate_to_width(&right_text, width as usize),
        Style::default().fg(TEXT_DIM).bg(BG),
        area.right(),
    );
}

fn render_confirm_quit(area: Rect, buf: &mut Buffer) {
    let message =
        " Unsaved changes. Ctrl+Q force quit  ·  Ctrl+S save & quit  ·  Esc cancel ";
    let _ = write_text(
        buf,
        area.left(),
        area.top(),
        &truncate_to_width(message, area.width as usize),
        Style::default().fg(WARN).bg(BG).add_modifier(Modifier::BOLD),
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
}
