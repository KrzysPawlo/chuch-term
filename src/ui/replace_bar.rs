use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;

fn put(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style, max_x: u16) -> u16 {
    let mut cx = x;
    for ch in text.chars() {
        if cx >= max_x { break; }
        buf[(cx, y)].set_char(ch).set_style(style);
        cx += 1;
    }
    cx
}

pub struct ReplaceBar<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for ReplaceBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 { return; }
        let bg = self.state.palette.theme_bg_bar;
        let accent_color = self.state.palette.theme_accent;
        let amber_color = self.state.palette.theme_warning;
        let dim_color = self.state.palette.theme_dim;

        let y = area.top();
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(bg).set_fg(dim_color).set_char(' ');
        }

        let accent_bold = Style::default().fg(accent_color).bg(bg).add_modifier(Modifier::BOLD);
        let accent = Style::default().fg(accent_color).bg(bg);
        let amber_bold = Style::default().fg(amber_color).bg(bg).add_modifier(Modifier::BOLD);
        let dim = Style::default().fg(dim_color).bg(bg);
        let sep = Style::default().fg(self.state.palette.hints_sep_fg).bg(bg);

        let max_x = area.right();
        let mut x = area.left() + 1;

        // "/ search_query"
        x = put(buf, x, y, "/ ", accent_bold, max_x);
        x = put(buf, x, y, &self.state.search_query, accent, max_x);

        // Case indicator
        if self.state.search_case_sensitive {
            x = put(buf, x, y, " [Cc]", amber_bold, max_x);
        }

        // " → "
        x = put(buf, x, y, "  ", dim, max_x);
        x = put(buf, x, y, "\u{2192}", dim, max_x);
        x = put(buf, x, y, "  ", dim, max_x);

        // replace_text + cursor
        x = put(buf, x, y, &self.state.replace_query, amber_bold, max_x);
        x = put(buf, x, y, "_", amber_bold, max_x);

        // match count
        let total = self.state.search_results.len();
        let current = if total == 0 { 0 } else { self.state.search_result_idx + 1 };
        let count = format!("   [{current}/{total}]");
        x = put(buf, x, y, &count, dim, max_x);

        // hints
        let hints = [
            ("Enter", " Replace"),
            ("^A", " All"),
            ("^N", " Next"),
            ("Esc", " Close"),
        ];
        for (i, (key, desc)) in hints.iter().enumerate() {
            if i > 0 {
                x = put(buf, x, y, "  \u{00b7}  ", sep, max_x);
            } else {
                x = put(buf, x, y, "   ", sep, max_x);
            }
            x = put(buf, x, y, key, accent_bold, max_x);
            x = put(buf, x, y, desc, dim, max_x);
        }
        let _ = x;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{EditorMode, EditorState};
    use ratatui::style::Color;

    #[test]
    fn replace_bar_uses_theme_bg_bar() {
        let mut state = EditorState::new_empty();
        state.mode = EditorMode::Replace;
        state.config.render.color_mode = "rgb".to_string();
        state.config.theme.bg_bar = "#667788".to_string();
        let config = state.config.clone();
        state.apply_config(config);

        let area = Rect::new(0, 0, 18, 1);
        let mut buf = Buffer::empty(area);
        ReplaceBar { state: &state }.render(area, &mut buf);

        for x in area.left()..area.right() {
            assert_eq!(buf[(x, 0)].bg, Color::Rgb(102, 119, 136));
        }
    }
}
