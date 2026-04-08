use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;

const SEP: Color = Color::Rgb(45, 45, 45);

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
        let (r, g, b) = self.state.config.theme.bg_bar_rgb();
        let bg = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.accent_rgb();
        let accent_color = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.warning_rgb();
        let amber_color = Color::Rgb(r, g, b);
        let (r, g, b) = self.state.config.theme.dim_rgb();
        let dim_color = Color::Rgb(r, g, b);

        let y = area.top();
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(bg).set_fg(dim_color).set_char(' ');
        }

        let accent_bold = Style::default().fg(accent_color).bg(bg).add_modifier(Modifier::BOLD);
        let accent = Style::default().fg(accent_color).bg(bg);
        let amber_bold = Style::default().fg(amber_color).bg(bg).add_modifier(Modifier::BOLD);
        let dim = Style::default().fg(dim_color).bg(bg);
        let sep = Style::default().fg(SEP).bg(bg);

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
