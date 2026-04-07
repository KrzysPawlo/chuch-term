use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;
use crate::commands::COMMANDS;

const OVERLAY_BG: Color = Color::Rgb(10, 10, 10);
const HEADER_FG: Color = Color::Rgb(176, 196, 200);   // #b0c4c8
const KEY_FG: Color = Color::Rgb(255, 153, 68);       // #ff9944
const DESC_FG: Color = Color::Rgb(90, 90, 90);        // #5a5a5a
const SEL_BG: Color = Color::Rgb(176, 196, 200);      // #b0c4c8
const SEL_FG: Color = Color::Rgb(10, 10, 10);         // #0a0a0a
const DIM: Color = Color::Rgb(50, 50, 50);

/// Column offset from the left margin where key hints are displayed.
const CMD_KEY_COL: u16 = 25;
/// Column offset from the left margin where descriptions are displayed.
const CMD_DESC_COL: u16 = 38;

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

/// Full-screen command palette overlay.
pub struct CommandPalette<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for CommandPalette<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Fill background
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_char(' ').set_bg(OVERLAY_BG).set_fg(OVERLAY_BG);
            }
        }

        if area.width < 40 || area.height < 5 {
            return;
        }

        let margin = area.left() + 3;
        let right = area.right().saturating_sub(3);
        let mut y = area.top() + 1;

        // ── Title ──────────────────────────────────────────────────────
        let title_style = Style::default()
            .fg(HEADER_FG)
            .bg(OVERLAY_BG)
            .add_modifier(Modifier::BOLD);
        put(buf, margin, y, "Command Palette", title_style, right);
        y += 1;

        // ── Query line ─────────────────────────────────────────────────
        let query_prefix = "> ";
        let query_style = Style::default().fg(HEADER_FG).bg(OVERLAY_BG);
        let mut qx = margin;
        qx = put(buf, qx, y, query_prefix, query_style, right);
        put(buf, qx, y, &self.state.palette_query, query_style, right);
        // cursor underscore
        let cursor_x = qx + self.state.palette_query.len() as u16;
        if cursor_x < right {
            buf[(cursor_x, y)].set_char('_').set_style(query_style);
        }
        y += 1;

        // ── Separator ─────────────────────────────────────────────────
        if y < area.bottom() {
            let sep_style = Style::default().fg(DIM).bg(OVERLAY_BG);
            for x in margin..right {
                buf[(x, y)].set_char('\u{2500}').set_style(sep_style);
            }
            y += 1;
        }

        // ── Command list ───────────────────────────────────────────────
        let visible_rows = area.bottom().saturating_sub(y + 1) as usize;
        let palette_cursor = self.state.palette_cursor;

        // Scroll so selected item is visible
        let scroll_offset = if palette_cursor >= visible_rows {
            palette_cursor - visible_rows + 1
        } else {
            0
        };

        for (display_row, &cmd_idx) in self
            .state
            .palette_matches
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_rows)
        {
            if y >= area.bottom().saturating_sub(1) {
                break;
            }
            let is_selected = display_row == palette_cursor;
            let cmd = &COMMANDS[cmd_idx];

            if is_selected {
                // Fill selected row background
                for x in margin..right {
                    buf[(x, y)].set_bg(SEL_BG).set_fg(SEL_FG);
                }
                let sel_name_style = Style::default().fg(SEL_FG).bg(SEL_BG).add_modifier(Modifier::BOLD);
                let sel_key_style = Style::default().fg(SEL_FG).bg(SEL_BG);
                let sel_desc_style = Style::default().fg(SEL_FG).bg(SEL_BG);

                let mut x = margin + 1;
                x = put(buf, x, y, cmd.name, sel_name_style, right);
                // Key hint right-aligned-ish
                let key_start = margin + CMD_KEY_COL;
                if key_start < right && !cmd.key.is_empty() {
                    put(buf, key_start, y, cmd.key, sel_key_style, right.saturating_sub(CMD_KEY_COL));
                }
                // Description
                let desc_start = margin + CMD_DESC_COL;
                if desc_start < right {
                    put(buf, desc_start, y, cmd.description, sel_desc_style, right);
                }
                let _ = x;
            } else {
                let name_style = Style::default().fg(HEADER_FG).bg(OVERLAY_BG);
                let key_style = Style::default().fg(KEY_FG).bg(OVERLAY_BG);
                let desc_style = Style::default().fg(DESC_FG).bg(OVERLAY_BG);

                let mut x = margin + 1;
                x = put(buf, x, y, cmd.name, name_style, right);
                let key_start = margin + CMD_KEY_COL;
                if key_start < right && !cmd.key.is_empty() {
                    put(buf, key_start, y, cmd.key, key_style, right.saturating_sub(CMD_KEY_COL));
                }
                let desc_start = margin + CMD_DESC_COL;
                if desc_start < right {
                    put(buf, desc_start, y, cmd.description, desc_style, right);
                }
                let _ = x;
            }

            y += 1;
        }

        // ── Footer ────────────────────────────────────────────────────
        {
            let fy = area.bottom().saturating_sub(1);
            let key_style = Style::default().fg(KEY_FG).bg(OVERLAY_BG).add_modifier(Modifier::BOLD);
            let desc_style = Style::default().fg(DESC_FG).bg(OVERLAY_BG);
            let sep_style = Style::default().fg(DIM).bg(OVERLAY_BG);

            // Build footer: "↑↓ Navigate  ·  Enter Execute  ·  Esc Close"
            let parts: &[(&str, &str)] = &[
                ("\u{2191}\u{2193}", " Navigate"),
                ("Enter", " Execute"),
                ("Esc", " Close"),
            ];
            // Measure total width
            let sep = "  \u{00b7}  ";
            let total_w: usize = parts.iter().map(|(k, d)| k.len() + d.len()).sum::<usize>()
                + sep.len() * (parts.len().saturating_sub(1));
            let start_x = area
                .left()
                .saturating_add(area.width.saturating_sub(total_w as u16) / 2);
            let mut fx = start_x;
            for (i, (key, desc)) in parts.iter().enumerate() {
                if i > 0 {
                    fx = put(buf, fx, fy, sep, sep_style, area.right());
                }
                fx = put(buf, fx, fy, key, key_style, area.right());
                fx = put(buf, fx, fy, desc, desc_style, area.right());
            }
            let _ = fx;
        }
    }
}
