use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use crate::editor::EditorState;

// ── Hardcoded (non-themed) design tokens ──────────────────────────────
const OVERLAY_BG: Color = Color::Rgb(10, 10, 10);
const SECTION_FG: Color = Color::Rgb(130, 130, 130);
const LABEL_FG:   Color = Color::Rgb(190, 190, 190);
const CHECK_ON:   Color = Color::Rgb(130, 200, 150);  // [x] checked — green
const CHECK_OFF:  Color = Color::Rgb(70,  70,  70);   // [ ] unchecked — dim
const DIM_FG:     Color = Color::Rgb(60,  60,  60);
const FOOTER_FG:  Color = Color::Rgb(50,  50,  50);

/// Number of interactive items in the settings list.
/// Must match the item indices in `toggle_setting()` in input/mod.rs.
pub const SETTINGS_ITEM_COUNT: usize = 9;

pub struct SettingsOverlay<'a> {
    pub state: &'a EditorState,
}

impl<'a> Widget for SettingsOverlay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Fill everything with overlay bg.
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_char(' ').set_bg(OVERLAY_BG).set_fg(OVERLAY_BG);
            }
        }

        let (r, g, b) = self.state.config.theme.accent_rgb();
        let accent = Color::Rgb(r, g, b);

        if area.width < 50 || area.height < 16 {
            render_compact(area, buf, accent);
            return;
        }

        render_full(self.state, area, buf, accent);
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

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

fn rule(buf: &mut Buffer, y: u16, x: u16, max_x: u16) -> u16 {
    let style = Style::default().fg(Color::Rgb(28, 28, 28)).bg(OVERLAY_BG);
    let mut cx = x;
    while cx < max_x {
        buf[(cx, y)].set_char('\u{2500}').set_style(style);
        cx += 1;
    }
    y.saturating_add(1)
}

/// Parameters for a single settings row.
struct ItemRow<'a> {
    label: &'a str,
    hint: Option<&'a str>,      // keyboard hint shown on right (e.g. "Ctrl+L")
    checked: Option<bool>,      // Some(bool) for checkboxes, None for non-bool
    value_str: Option<&'a str>, // Some("4") for numeric / enum items
}

/// Shared layout context for item_row calls.
struct RowCtx {
    left: u16,
    right: u16,
    cursor: usize,
    accent: Color,
}

/// Draw a settings row and return the next y.
fn item_row(buf: &mut Buffer, y: u16, ctx: &RowCtx, idx: usize, row: ItemRow<'_>) -> u16 {
    let is_selected = ctx.cursor == idx;
    let left = ctx.left;
    let right = ctx.right;
    let accent = ctx.accent;

    // Arrow or space prefix.
    let prefix = if is_selected { "\u{25ba} " } else { "  " }; // ► or spaces
    let prefix_style = Style::default()
        .fg(if is_selected { accent } else { OVERLAY_BG })
        .bg(OVERLAY_BG)
        .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() });
    let mut x = put(buf, left, y, prefix, prefix_style, right);

    // Checkbox or indent.
    match row.checked {
        Some(true) => {
            x = put(buf, x, y, "[", Style::default().fg(CHECK_ON).bg(OVERLAY_BG), right);
            x = put(buf, x, y, "x", Style::default().fg(CHECK_ON).bg(OVERLAY_BG).add_modifier(Modifier::BOLD), right);
            x = put(buf, x, y, "] ", Style::default().fg(CHECK_ON).bg(OVERLAY_BG), right);
        }
        Some(false) => {
            x = put(buf, x, y, "[ ] ", Style::default().fg(CHECK_OFF).bg(OVERLAY_BG), right);
        }
        None => {
            x = put(buf, x, y, "    ", Style::default().fg(OVERLAY_BG).bg(OVERLAY_BG), right);
        }
    }

    // Label.
    let label_style = Style::default()
        .fg(if is_selected { LABEL_FG } else { Color::Rgb(150, 150, 150) })
        .bg(OVERLAY_BG);
    x = put(buf, x, y, row.label, label_style, right);

    // Value (for numeric / enum).
    if let Some(val) = row.value_str {
        x = put(buf, x, y, ": ", Style::default().fg(DIM_FG).bg(OVERLAY_BG), right);
        x = put(buf, x, y, val, Style::default().fg(accent).bg(OVERLAY_BG).add_modifier(Modifier::BOLD), right);
    }

    // Right-aligned hint.
    if let Some(hint_text) = row.hint {
        let hint_x = right.saturating_sub(hint_text.chars().count() as u16 + 1);
        if hint_x > x {
            put(buf, hint_x, y, hint_text, Style::default().fg(DIM_FG).bg(OVERLAY_BG), right);
        }
    }

    y + 1
}

// ── Full layout ────────────────────────────────────────────────────────

fn render_full(state: &EditorState, area: Rect, buf: &mut Buffer, accent: Color) {
    let margin = area.left() + 3;
    let right = area.right().saturating_sub(3);
    let mut y = area.top() + 1;
    let cfg = &state.config;
    let cur = state.settings_cursor;

    // ── Header ────────────────────────────────────────────────────────
    let header_style = Style::default()
        .fg(accent)
        .bg(OVERLAY_BG)
        .add_modifier(Modifier::BOLD);
    put(buf, margin, y, "Settings", header_style, right);
    let close_hint = "Esc to close";
    let close_x = right.saturating_sub(close_hint.chars().count() as u16);
    put(buf, close_x, y, close_hint, Style::default().fg(FOOTER_FG).bg(OVERLAY_BG), area.right());
    y += 1;
    y += 1;
    y = rule(buf, y, margin, right);
    y += 1;

    // ── EDITOR section ────────────────────────────────────────────────
    put(buf, margin, y, "EDITOR", Style::default().fg(SECTION_FG).bg(OVERLAY_BG).add_modifier(Modifier::BOLD), right);
    y += 1;

    let tab_w_str = cfg.editor.tab_width.to_string();
    let ctx = RowCtx { left: margin, right, cursor: cur, accent };
    y = item_row(buf, y, &ctx, 0, ItemRow { label: "Line numbers",          hint: Some("Ctrl+L"),            checked: Some(cfg.editor.line_numbers),    value_str: None });
    y = item_row(buf, y, &ctx, 1, ItemRow { label: "Relative numbers",      hint: None,                      checked: Some(cfg.editor.relative_numbers), value_str: None });
    y = item_row(buf, y, &ctx, 2, ItemRow { label: "Syntax highlighting",   hint: None,                      checked: Some(cfg.editor.syntax_highlight), value_str: None });
    y = item_row(buf, y, &ctx, 3, ItemRow { label: "Auto-indent on Enter",  hint: None,                      checked: Some(cfg.editor.auto_indent),      value_str: None });
    y = item_row(buf, y, &ctx, 4, ItemRow { label: "Expand tabs to spaces", hint: None,                      checked: Some(cfg.editor.expand_tabs),      value_str: None });
    y = item_row(buf, y, &ctx, 5, ItemRow { label: "Tab width",             hint: Some("\u{2190} \u{2192}"), checked: None,                              value_str: Some(&tab_w_str) });
    y = item_row(buf, y, &ctx, 6, ItemRow { label: "Indent guides",         hint: None,                      checked: Some(cfg.editor.indent_guides),    value_str: None });
    y = item_row(buf, y, &ctx, 7, ItemRow { label: "Indent error hints",    hint: None,                      checked: Some(cfg.editor.indent_errors),    value_str: None });
    y += 1;

    // ── CLIPBOARD section ─────────────────────────────────────────────
    if y + 3 < area.bottom() {
        put(buf, margin, y, "CLIPBOARD", Style::default().fg(SECTION_FG).bg(OVERLAY_BG).add_modifier(Modifier::BOLD), right);
        y += 1;
        y = item_row(buf, y, &ctx, 8, ItemRow { label: "Strategy", hint: Some("\u{2190} \u{2192}"), checked: None, value_str: Some(&cfg.clipboard.strategy) });
        y += 1;
    }

    // ── Footer ────────────────────────────────────────────────────────
    if y + 2 < area.bottom() {
        y = rule(buf, y, margin, right);
        let note = "Changes saved to config.toml on close";
        put(buf, margin, y, note, Style::default().fg(FOOTER_FG).bg(OVERLAY_BG), right);
    }
}

// ── Compact (narrow terminal) ──────────────────────────────────────────

fn render_compact(area: Rect, buf: &mut Buffer, accent: Color) {
    let msg = "Settings  \u{2014}  Esc to close";
    let msg_len = msg.chars().count() as u16;
    let y = area.top() + area.height / 2;
    let x = area.left().saturating_add(area.width.saturating_sub(msg_len) / 2);
    let style = Style::default().fg(accent).bg(OVERLAY_BG);
    let mut cx = x;
    for ch in msg.chars() {
        if cx >= area.right() { break; }
        buf[(cx, y)].set_char(ch).set_style(style);
        cx += 1;
    }
}
