use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;

const BG: Color = Color::Rgb(18, 18, 18);
const ACCENT: Color = Color::Rgb(176, 196, 200);   // #b0c4c8
const DIM: Color = Color::Rgb(90, 90, 90);          // #5a5a5a
const AMBER: Color = Color::Rgb(255, 153, 68);      // #ff9944
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
        let y = area.top();
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(BG).set_char(' ');
        }

        let accent_bold = Style::default().fg(ACCENT).bg(BG).add_modifier(Modifier::BOLD);
        let accent = Style::default().fg(ACCENT).bg(BG);
        let amber_bold = Style::default().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD);
        let dim = Style::default().fg(DIM).bg(BG);
        let sep = Style::default().fg(SEP).bg(BG);

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
