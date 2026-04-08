use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crate::editor::EditorState;
use crate::syntax::TokenKind;

fn token_color(palette: &crate::color::Palette, kind: TokenKind) -> Color {
    match kind {
        TokenKind::Keyword => palette.syntax_keyword_fg,
        TokenKind::String => palette.syntax_string_fg,
        TokenKind::Comment => palette.syntax_comment_fg,
        TokenKind::Number => palette.syntax_number_fg,
        TokenKind::Type => palette.syntax_type_fg,
        TokenKind::Attribute => palette.syntax_attribute_fg,
    }
}

/// Renders the text editing area: visible buffer lines + cursor placement.
pub struct EditorView<'a> {
    pub state: &'a EditorState,
}

impl<'a> EditorView<'a> {
    /// Calculate the display column of the cursor for correct horizontal placement.
    pub fn cursor_display_col(state: &EditorState) -> u16 {
        let line = state.buffer.line(state.cursor.row);
        let mut display_col: u16 = 0;
        let target = state.cursor.col.min(line.len());
        for (byte_idx, grapheme) in line.grapheme_indices(true) {
            if byte_idx >= target {
                break;
            }
            display_col += grapheme.width() as u16;
        }
        display_col
    }
}

impl<'a> Widget for EditorView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        if self.state.is_welcome_state() {
            render_welcome(self.state, area, buf);
            return;
        }

        render_buffer(self.state, area, buf);
    }
}

// ── Buffer rendering ───────────────────────────────────────────────────

fn render_buffer(state: &EditorState, area: Rect, buf: &mut Buffer) {
    let offset = state.viewport.offset_row;
    let line_count = state.buffer.line_count();
    let width = area.width as usize;
    let lang = state.language();
    let syntax_enabled = state.config.editor.syntax_highlight;
    let search_current_bg = state.palette.theme_accent;

    // Pre-compute selection range
    let sel_range = state.selection_range();

    for screen_row in 0..area.height {
        let buf_row = offset + screen_row as usize;
        let y = area.top() + screen_row;

        if buf_row < line_count {
            let line = state.buffer.line(buf_row).to_string();

            // Get syntax tokens for this line
            let tokens: Vec<(usize, usize, TokenKind)> = if syntax_enabled {
                crate::syntax::highlight_line(&line, lang)
                    .into_iter()
                    .map(|t| (t.start, t.end, t.kind))
                    .collect()
            } else {
                Vec::new()
            };

            // Pre-compute leading-whitespace byte extent.
            let leading_ws_end: usize = line
                .char_indices()
                .take_while(|(_, c)| *c == ' ' || *c == '\t')
                .last()
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(0);

            // Check if this line has an indentation error (YAML / Python / Proto).
            let tab_width = state.config.editor.tab_width;
            let line_has_error = state.config.editor.indent_errors
                && crate::syntax::has_indent_error(&line, tab_width, lang);
            let error_bg_color = crate::color::resolve_config_rgb(
                state.render_decision.effective,
                state.config.editor.indent_error_bg,
            );
            let guides_on = state.config.editor.indent_guides;

            let mut x = area.left();
            let mut display_cols = 0usize;
            let mut byte_pos = 0usize;

            for ch in line.chars() {
                let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);
                if display_cols + ch_width > width || x >= area.right() {
                    break;
                }

                let in_leading_ws = byte_pos < leading_ws_end;

                // Determine base style from syntax
                let mut fg = state.palette.text_main;
                let mut bg = state.palette.editor_bg;
                let mods = Modifier::empty();

                if syntax_enabled {
                    // Find the token covering this byte position
                    for &(start, end, kind) in &tokens {
                        if byte_pos >= start && byte_pos < end {
                            fg = token_color(&state.palette, kind);
                            break;
                        }
                    }
                }

                // Indentation error background (applied before selection so errors are visible).
                if line_has_error && in_leading_ws {
                    bg = error_bg_color;
                }

                // Check selection highlight
                if let Some((sel_start, sel_end)) = sel_range {
                    let in_sel = is_in_selection(buf_row, byte_pos, sel_start, sel_end);
                    if in_sel {
                        bg = state.palette.selection_bg;
                    }
                }

                // Check search match highlight (overrides selection for matches)
                if !state.search_query.is_empty() {
                    for (match_idx, found) in state.search_results.iter().enumerate() {
                        if found.row == buf_row
                            && byte_pos >= found.start
                            && byte_pos < found.end
                        {
                            if match_idx == state.search_result_idx {
                                bg = search_current_bg;
                                fg = state.palette.search_current_fg;
                            } else {
                                bg = state.palette.search_match_bg;
                            }
                            break;
                        }
                    }
                }

                // Indent guide: replace a space at a tab-stop column with '│'.
                let is_guide = guides_on
                    && in_leading_ws
                    && display_cols > 0
                    && tab_width > 0
                    && display_cols % tab_width as usize == 0
                    && (ch == ' ' || ch == '\t');

                let render_ch = if is_guide { '\u{2502}' } else { ch }; // '│'
                let final_fg = if is_guide { state.palette.indent_guide_fg } else { fg };
                let style = Style::default().fg(final_fg).bg(bg).add_modifier(mods);
                buf[(x, y)].set_char(render_ch).set_style(style);
                x += 1;

                if ch_width == 2 && x < area.right() {
                    buf[(x, y)].set_char(' ').set_style(style);
                    x += 1;
                }

                display_cols += ch_width;
                byte_pos += ch.len_utf8();
            }

            // Clear remainder of line with explicit background to avoid ghost
            // selection artifacts when text is deleted or selection is cleared.
            while x < area.right() {
                buf[(x, y)].set_char(' ').set_style(Style::default().bg(state.palette.editor_bg));
                x += 1;
            }
        } else {
            buf[(area.left(), y)]
                .set_char('~')
                .set_style(Style::default().fg(state.palette.tilde_fg).bg(state.palette.editor_bg));
            let mut x = area.left() + 1;
            while x < area.right() {
                buf[(x, y)].set_char(' ').set_style(Style::default().bg(state.palette.editor_bg));
                x += 1;
            }
        }
    }
}

/// Check if (row, byte_col) is within the selection [sel_start, sel_end).
fn is_in_selection(
    row: usize,
    col: usize,
    sel_start: (usize, usize),
    sel_end: (usize, usize),
) -> bool {
    let (sr, sc) = sel_start;
    let (er, ec) = sel_end;
    if row < sr || row > er {
        return false;
    }
    if row == sr && row == er {
        return col >= sc && col < ec;
    }
    if row == sr {
        return col >= sc;
    }
    if row == er {
        return col < ec;
    }
    true // middle rows fully selected
}

// ── Welcome screen ─────────────────────────────────────────────────────

fn render_welcome(state: &EditorState, area: Rect, buf: &mut Buffer) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf[(x, y)].set_char(' ').set_style(
                Style::default().bg(state.palette.editor_bg).fg(state.palette.editor_bg),
            );
        }
    }

    if area.height < 6 || area.width < 20 {
        return;
    }

    let welcome_name_fg = state.palette.theme_accent;
    let welcome_key_fg = state.palette.theme_warning;

    let block_height: u16 = 4;
    let center_y = area.top() + area.height / 2;
    let start_y = center_y.saturating_sub(block_height / 2);

    let name_line = "chuch-term";

    let hints: &[(&str, &str)] = &[
        ("^H", "help"),
        ("^Q", "quit"),
        ("^S", "save"),
    ];

    let name_style = Style::default()
        .fg(welcome_name_fg)
        .add_modifier(Modifier::BOLD);
    render_centered(buf, start_y, area, name_line, name_style);

    let hints_y = start_y + 2;
    if hints_y < area.bottom() {
        let sep = "  \u{00b7}  ";
        let hint_items: Vec<String> = hints.iter().map(|(k, d)| format!("{k}  {d}")).collect();
        let total_len: usize = hint_items.iter().map(|s| s.len()).sum::<usize>()
            + sep.len() * (hint_items.len().saturating_sub(1));

        let start_x = area
            .left()
            .saturating_add(area.width.saturating_sub(total_len as u16) / 2);
        let mut x = start_x;
        let max_x = area.right();

        let key_style = Style::default().fg(welcome_key_fg);
        let desc_style = Style::default().fg(state.palette.settings_dim_fg);
        let sep_style = Style::default().fg(state.palette.overlay_rule_fg);

        for (i, (key, desc)) in hints.iter().enumerate() {
            if i > 0 {
                x = write_str(buf, x, hints_y, sep, sep_style, max_x);
            }
            x = write_str(buf, x, hints_y, key, key_style, max_x);
            x = write_str(buf, x, hints_y, "  ", desc_style, max_x);
            x = write_str(buf, x, hints_y, desc, desc_style, max_x);
        }
        let _ = x;
    }
}

fn render_centered(buf: &mut Buffer, y: u16, area: Rect, text: &str, style: Style) {
    if y >= area.bottom() {
        return;
    }
    let text_len = text.chars().count() as u16;
    let x = area
        .left()
        .saturating_add(area.width.saturating_sub(text_len) / 2);
    write_str(buf, x, y, text, style, area.right());
}

fn write_str(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style, max_x: u16) -> u16 {
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
