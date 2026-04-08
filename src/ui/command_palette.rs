use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;
use crate::commands::COMMANDS;

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
        // Resolve theme colours.
        let accent = self.state.palette.theme_accent;
        let key_color = self.state.palette.theme_warning;
        let desc_color = self.state.palette.theme_dim;
        let overlay_bg = self.state.palette.overlay_bg;
        let selected_fg = self.state.palette.command_selected_fg;
        let separator_fg = self.state.palette.command_separator_fg;

        // Fill background
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_char(' ').set_bg(overlay_bg).set_fg(overlay_bg);
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
            .fg(accent)
            .bg(overlay_bg)
            .add_modifier(Modifier::BOLD);
        put(buf, margin, y, "Command Palette", title_style, right);
        y += 1;

        // ── Query line ─────────────────────────────────────────────────
        let query_style = Style::default().fg(accent).bg(overlay_bg);
        let mut qx = margin;
        qx = put(buf, qx, y, "> ", query_style, right);
        put(buf, qx, y, &self.state.palette_query, query_style, right);
        let cursor_x = qx + self.state.palette_query.len() as u16;
        if cursor_x < right {
            buf[(cursor_x, y)].set_char('_').set_style(query_style);
        }
        y += 1;

        // ── Separator ─────────────────────────────────────────────────
        if y < area.bottom() {
            let sep_style = Style::default().fg(separator_fg).bg(overlay_bg);
            for x in margin..right {
                buf[(x, y)].set_char('\u{2500}').set_style(sep_style);
            }
            y += 1;
        }

        // ── Command list ───────────────────────────────────────────────
        let visible_rows = area.bottom().saturating_sub(y + 1) as usize;
        let palette_cursor = self.state.palette_cursor;

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
                for x in margin..right {
                    buf[(x, y)].set_bg(accent).set_fg(selected_fg);
                }
                let sel_style = Style::default().fg(selected_fg).bg(accent);
                let sel_bold  = Style::default().fg(selected_fg).bg(accent).add_modifier(Modifier::BOLD);

                let mut x = margin + 1;
                x = put(buf, x, y, cmd.name, sel_bold, right);
                let key_start = margin + CMD_KEY_COL;
                if key_start < right && !cmd.key.is_empty() {
                    put(buf, key_start, y, cmd.key, sel_style, right.saturating_sub(CMD_KEY_COL));
                }
                let desc_start = margin + CMD_DESC_COL;
                if desc_start < right {
                    put(buf, desc_start, y, cmd.description, sel_style, right);
                }
                let _ = x;
            } else {
                let name_style = Style::default().fg(accent).bg(overlay_bg);
                let key_style  = Style::default().fg(key_color).bg(overlay_bg);
                let desc_style = Style::default().fg(desc_color).bg(overlay_bg);

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
            let fkey_style  = Style::default().fg(key_color).bg(overlay_bg).add_modifier(Modifier::BOLD);
            let fdesc_style = Style::default().fg(desc_color).bg(overlay_bg);
            let sep_style   = Style::default().fg(separator_fg).bg(overlay_bg);

            let parts: &[(&str, &str)] = &[
                ("\u{2191}\u{2193}", " Navigate"),
                ("Enter", " Execute"),
                ("Esc", " Close"),
            ];
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
                fx = put(buf, fx, fy, key, fkey_style, area.right());
                fx = put(buf, fx, fy, desc, fdesc_style, area.right());
            }
            let _ = fx;
        }
    }
}
