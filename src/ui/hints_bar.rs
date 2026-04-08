use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::{EditorMode, EditorState};

// ── Design tokens ──────────────────────────────────────────────────────
const BG: Color = Color::Rgb(18, 18, 18);           // #121212
const KEY_FG: Color = Color::Rgb(176, 196, 200);    // #b0c4c8  accent
const DESC_FG: Color = Color::Rgb(90, 90, 90);      // #5a5a5a  dim description
const SEP_FG: Color = Color::Rgb(45, 45, 45);       // #2d2d2d  separator ·
const WARN_FG: Color = Color::Rgb(255, 153, 68);    // #ff9944  amber for confirm state
const WARN_SEP: Color = Color::Rgb(100, 60, 20);    // dim amber separator

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

        // Fill background.
        for x in area.left()..area.right() {
            buf[(x, y)].set_bg(BG).set_char(' ');
        }

        match self.state.mode {
            EditorMode::Normal => {
                if self.state.selection_anchor.is_some() {
                    render_selection_hints(area, buf);
                } else {
                    render_normal(area, buf, self.state.previous_buffer.is_some());
                }
            }
            EditorMode::ConfirmQuit => render_confirm(area, buf),
            EditorMode::Help => render_help(area, buf),
            EditorMode::Search | EditorMode::GoToLine => {
                // These modes render their own bar widget (search_bar / goto_bar)
                // This fallback should not be reached since mod.rs dispatches them,
                // but keep it safe.
            }
            EditorMode::Replace => {
                // ReplaceBar widget handles this — nothing in hints bar
            }
            EditorMode::CommandPalette => {
                // Palette overlay handles everything — nothing here
            }
            EditorMode::SaveAs => {
                // SaveAsBar widget handles this — nothing in hints bar
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

fn render_normal(area: Rect, buf: &mut Buffer, has_prev: bool) {
    let key_style = Style::default().fg(KEY_FG).bg(BG).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(DESC_FG).bg(BG);
    let sep_style = Style::default().fg(SEP_FG).bg(BG);

    if has_prev {
        render_hints(
            area,
            buf,
            &[
                ("^S", "Save"),
                ("^Z", "Undo"),
                ("^F", "Find"),
                ("^P", "Commands"),
                ("^O", "Back"),
                ("^H", "Help"),
            ],
            key_style,
            desc_style,
            sep_style,
        );
    } else {
        render_hints(
            area,
            buf,
            &[
                ("^S", "Save"),
                ("^Z", "Undo"),
                ("^F", "Find"),
                ("^P", "Commands"),
                ("^H", "Help"),
            ],
            key_style,
            desc_style,
            sep_style,
        );
    }
}

fn render_selection_hints(area: Rect, buf: &mut Buffer) {
    let key_style = Style::default().fg(KEY_FG).bg(BG).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(DESC_FG).bg(BG);
    let sep_style = Style::default().fg(SEP_FG).bg(BG);

    render_hints(
        area,
        buf,
        &[
            ("^C", "Copy"),
            ("^X", "Cut"),
            ("^V", "Paste"),
            ("Esc", "Clear"),
        ],
        key_style,
        desc_style,
        sep_style,
    );
}

fn render_confirm(area: Rect, buf: &mut Buffer) {
    let key_style = Style::default().fg(WARN_FG).bg(BG).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(WARN_FG).bg(BG);
    let sep_style = Style::default().fg(WARN_SEP).bg(BG);

    render_hints(
        area,
        buf,
        &[
            ("^Q", "Force Quit"),
            ("^S", "Save & Quit"),
            ("Esc", "Cancel"),
        ],
        key_style,
        desc_style,
        sep_style,
    );
}

fn render_help(area: Rect, buf: &mut Buffer) {
    let text = "Esc  Close Help";
    let text_len = text.chars().count() as u16;
    let x = area
        .left()
        .saturating_add(area.width.saturating_sub(text_len) / 2);
    let style = Style::default().fg(SEP_FG).bg(BG);
    put(buf, x, area.top(), text, style, area.right());
}
