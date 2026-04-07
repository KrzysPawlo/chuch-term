use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

// ── Design tokens ──────────────────────────────────────────────────────
const OVERLAY_BG: Color = Color::Rgb(10, 10, 10);      // #0a0a0a  deep cosmic black
const HEADER_FG: Color = Color::Rgb(176, 196, 200);    // #b0c4c8  accent — product name
const VERSION_FG: Color = Color::Rgb(50, 50, 50);      // #323232  very dim version
const SECTION_FG: Color = Color::Rgb(130, 130, 130);   // #828282  section headers
const KEY_FG: Color = Color::Rgb(255, 153, 68);        // #ff9944  amber keys
const DESC_FG: Color = Color::Rgb(190, 190, 190);      // #bebebe  descriptions
const RULE_FG: Color = Color::Rgb(32, 32, 32);         // #202020  separator lines
const FOOTER_FG: Color = Color::Rgb(50, 50, 50);       // #323232  footer hint

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Full-screen help overlay. Rendered on top of everything when mode == Help.
pub struct HelpOverlay;

impl Widget for HelpOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Fill entire area with overlay background first — this covers all existing cells.
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)]
                    .set_char(' ')
                    .set_bg(OVERLAY_BG)
                    .set_fg(OVERLAY_BG);
            }
        }

        if area.width < 50 || area.height < 12 {
            render_compact(area, buf);
            return;
        }

        render_full(area, buf);
    }
}

// ── Render helpers ─────────────────────────────────────────────────────

/// Write text at (x, y) with style, clipped to max_x. Returns new x.
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

/// Draw a horizontal rule from x to max_x at row y. Returns next y.
fn rule(buf: &mut Buffer, y: u16, x: u16, max_x: u16) -> u16 {
    let style = Style::default().fg(RULE_FG).bg(OVERLAY_BG);
    let mut cx = x;
    while cx < max_x {
        buf[(cx, y)].set_char('\u{2500}').set_style(style); // ─
        cx += 1;
    }
    y.saturating_add(1)
}

/// Styles bundle for `kv()` to stay under clippy's argument-count limit.
struct KvStyles {
    key_w: u16,
    key: Style,
    desc: Style,
}

/// Draw a key+description pair. Key occupies `styles.key_w` columns, desc follows.
fn kv(buf: &mut Buffer, y: u16, x: u16, max_x: u16, key: &str, desc: &str, styles: &KvStyles) {
    let mut cx = x;
    // Write key, left-aligned within key_w.
    for ch in key.chars() {
        if cx >= x + styles.key_w || cx >= max_x {
            break;
        }
        buf[(cx, y)].set_char(ch).set_style(styles.key);
        cx += 1;
    }
    // Pad remaining key column with spaces.
    let bg_style = Style::default().bg(OVERLAY_BG).fg(OVERLAY_BG);
    while cx < x + styles.key_w && cx < max_x {
        buf[(cx, y)].set_char(' ').set_style(bg_style);
        cx += 1;
    }
    // Write description.
    for ch in desc.chars() {
        if cx >= max_x {
            break;
        }
        buf[(cx, y)].set_char(ch).set_style(styles.desc);
        cx += 1;
    }
}

// ── Full layout (≥ 50 cols, ≥ 12 rows) ────────────────────────────────

fn render_full(area: Rect, buf: &mut Buffer) {
    let margin = area.left() + 3;
    let right = area.right().saturating_sub(3);
    let mut y = area.top() + 1;

    // ── Header ────────────────────────────────────────────────────────
    let header_style = Style::default()
        .fg(HEADER_FG)
        .bg(OVERLAY_BG)
        .add_modifier(Modifier::BOLD);

    let hx = put(buf, margin, y, "chuch-term", header_style, right);
    // Version right-aligned on the same row.
    let ver = format!("v{VERSION}");
    let ver_x = right.saturating_sub(ver.len() as u16);
    if ver_x > hx {
        put(buf, ver_x, y, &ver, Style::default().fg(VERSION_FG).bg(OVERLAY_BG), right);
    }
    y += 1;
    y += 1;

    y = rule(buf, y, margin, right);
    y += 1;

    // ── Two-column keybindings ─────────────────────────────────────────
    let half = (area.width.saturating_sub(6)) / 2;
    let col1 = margin;
    let col2 = margin + half;
    const KEY_W: u16 = 12;

    let sec_style = Style::default()
        .fg(SECTION_FG)
        .bg(OVERLAY_BG)
        .add_modifier(Modifier::BOLD);
    let kv_styles = KvStyles {
        key_w: KEY_W,
        key: Style::default().fg(KEY_FG).bg(OVERLAY_BG).add_modifier(Modifier::BOLD),
        desc: Style::default().fg(DESC_FG).bg(OVERLAY_BG),
    };

    // Section headers.
    put(buf, col1, y, "NAVIGATION", sec_style, col2);
    put(buf, col2, y, "EDITING & FILE", sec_style, right);
    y += 1;

    // Paired rows: (left_key, left_desc, right_key, right_desc).
    // Empty string = skip that cell.
    let rows: &[(&str, &str, &str, &str)] = &[
        ("\u{2191}\u{2193}\u{2190}\u{2192}", "Move cursor",    "Type",          "Insert character"),
        ("Home",          "Line start",      "Backspace",      "Delete backward"),
        ("End",           "Line end",        "Delete",         "Delete forward"),
        ("PgUp",          "Scroll up",       "Enter",          "New line"),
        ("PgDn",          "Scroll down",     "Ctrl+S",         "Save file"),
        ("Shift+Arrows",  "Select text",     "Ctrl+Q",         "Quit"),
        ("Ctrl+\u{2190}/\u{2192}", "Word left/right", "Ctrl+H", "This help"),
        ("Ctrl+Z",        "Undo",            "Ctrl+F",         "Find in file"),
        ("Ctrl+Y",        "Redo",            "Ctrl+L",         "Toggle line numbers"),
        ("Ctrl+G",        "Go to line",      "Ctrl+C/X/V",     "Copy/Cut/Paste"),
        ("Ctrl+P",        "Command palette", "Ctrl+R",         "Find & replace"),
        ("Ctrl+A",        "Select all",      "Ctrl+O",         "Go back (prev file)"),
        ("Alt+U / Alt+L", "Upper/Lowercase", "Ctrl+W / Del",   "Delete word \u{2190}/\u{2192}"),
    ];

    for (k1, d1, k2, d2) in rows {
        if !k1.is_empty() {
            kv(buf, y, col1, col2, k1, d1, &kv_styles);
        }
        if !k2.is_empty() {
            kv(buf, y, col2, right, k2, d2, &kv_styles);
        }
        y += 1;
    }

    y += 1;

    // ── Footer ────────────────────────────────────────────────────────
    if y + 2 < area.bottom() {
        y = rule(buf, y, margin, right);
        let footer = "Esc or ^H to close";
        let footer_x = right.saturating_sub(footer.len() as u16);
        put(
            buf,
            footer_x,
            y,
            footer,
            Style::default().fg(FOOTER_FG).bg(OVERLAY_BG),
            area.right(),
        );
    }
}

// ── Compact layout (narrow / short terminal) ──────────────────────────

fn render_compact(area: Rect, buf: &mut Buffer) {
    let y = area.top() + area.height / 2;
    let msg = "chuch-term  \u{2014}  Esc to close help";
    let msg_len = msg.chars().count() as u16;
    let x = area
        .left()
        .saturating_add(area.width.saturating_sub(msg_len) / 2);
    put(
        buf,
        x,
        y,
        msg,
        Style::default().fg(HEADER_FG).bg(OVERLAY_BG),
        area.right(),
    );
}
