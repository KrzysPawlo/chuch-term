# chuch-term

![version](https://img.shields.io/badge/version-0.5.4-b0c4c8)
![license](https://img.shields.io/badge/license-MIT-green)
![rust](https://img.shields.io/badge/rust-1.78+-orange)

A fast, minimal terminal text editor — built in Rust, works everywhere.

```bash
chuch-term file.rs       # edit a file
chuch-term               # start with empty buffer
```

---

## Why chuch-term?

Most terminal editors have one of two problems: they're too minimal (nano — no undo, no search-and-replace, cryptic shortcuts) or too complex (vim/emacs — steep learning curve, modal editing, plugin ecosystems).

chuch-term hits the middle ground: **everything you need for daily editing**, nothing you don't.

- **Zero config required** — works out of the box with sensible defaults
- **Discoverable** — shortcuts shown at the bottom, help overlay on `Ctrl+H`
- **No modal editing** — just type, no mode switching to insert text
- **Runs anywhere** — single binary, macOS/Linux/Raspberry Pi/any server with a shell

---

## Installation

### Homebrew (coming soon)

```bash
brew install chuch-term   # not yet available — planned
```

### Pre-built binary

Download from [GitHub Releases](https://github.com/KrzysPawlo/chuch-term/releases/latest):

| Platform | File |
|----------|------|
| macOS Apple Silicon | `chuch-term-macos-arm.tar.gz` |
| macOS Intel | `chuch-term-macos-intel.tar.gz` |
| Linux x86_64 (static) | `chuch-term-linux-x86_64.tar.gz` |

```bash
# 1. Download and extract
#    Safari auto-unpacks .gz and saves a plain .tar — use tar xf (not tar xzf)
tar xf chuch-term-macos-arm.tar        # adjust filename for your platform

# 2. Verify checksum (download the matching .sha256 file from the same release)
shasum -a 256 -c chuch-term-macos-arm.tar.gz.sha256

# 3. Move to PATH
sudo mv chuch-term /usr/local/bin/

# 4. macOS Gatekeeper — required for every binary downloaded from the internet
xattr -d com.apple.quarantine /usr/local/bin/chuch-term

# 5. Verify
chuch-term --version
```

> **Why `xattr`?** macOS quarantines every binary downloaded from the internet.
> This one-time command removes the quarantine flag for that binary.
> To skip `xattr` and `sudo` entirely: use `cargo install --git` — Rust compiles
> locally so Gatekeeper never triggers and no root access is needed.

**No-sudo alternative** — install to your user directory:

```bash
mkdir -p ~/.local/bin
mv chuch-term ~/.local/bin/
xattr -d com.apple.quarantine ~/.local/bin/chuch-term   # macOS only
# Add to ~/.zshrc if not already present:
# export PATH="$HOME/.local/bin:$PATH"
```

### cargo install

```bash
cargo install --git https://github.com/KrzysPawlo/chuch-term
```

Get Rust: [rustup.rs](https://rustup.rs)

### Build from source

```bash
cargo build --release
cp target/release/chuch-term /usr/local/bin/
```

---

## What gets installed

| What | Where |
|------|-------|
| Binary | `/usr/local/bin/chuch-term` (you place it there manually) |
| Config | `~/.config/chuch-term/config.toml` (created automatically on first run) |

Nothing else — no background services, no shell hooks, no system-level changes.

---

## Update

Download the new release from [GitHub Releases](https://github.com/KrzysPawlo/chuch-term/releases/latest), then run the same steps as installation:

```bash
tar xf chuch-term-macos-arm.tar          # adjust for your platform
sudo mv chuch-term /usr/local/bin/
xattr -d com.apple.quarantine /usr/local/bin/chuch-term
chuch-term --version
```

The `xattr` step is required on every update — macOS re-quarantines each new download.

---

## Uninstall

```bash
chuch-term --uninstall
```

Removes the binary and `~/.config/chuch-term/`. Nothing else was ever installed.

---

## Features

- **Syntax highlighting** — Rust, Python, JavaScript/TypeScript, Go, TOML, YAML, Shell, Markdown, Proto3
- **Find & Replace** — incremental search with live match count, replace one or all
- **Undo / Redo** — unlimited history with smart word-level coalescing
- **Text selection** — Shift+arrows, Ctrl+A; Copy/Cut/Paste with system clipboard
- **Case tools** — UPPER and lower case on selection (Alt+U/L)
- **Line numbers** — absolute and relative, toggle with Ctrl+L
- **Go to line** — Ctrl+G, type a number, Enter
- **Command palette** — Ctrl+P, type any command name to find and execute it
- **Config file** — `~/.config/chuch-term/config.toml`, hot-reloaded within 2s
- **Single binary** — no runtime, no dependencies, copy the binary and go
- **Atomic saves** — tmp→rename pattern, no data loss on crash

---

## Keybindings

### Navigation
| Key | Action |
|-----|--------|
| `↑ ↓ ← →` | Move cursor |
| `Home` / `End` | Start / end of line |
| `PgUp` / `PgDn` | Scroll page |
| `Ctrl+←` / `Ctrl+→` | Jump word left / right |
| `Ctrl+G` | Go to line number |

### Editing
| Key | Action |
|-----|--------|
| Type | Insert text |
| `Backspace` / `Delete` | Delete character |
| `Enter` | New line |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+W` | Delete word before cursor |
| `Ctrl+Delete` | Delete word after cursor |
| `Alt+U` | UPPERCASE selection |
| `Alt+L` | lowercase selection |

### Find & Replace
| Key | Action |
|-----|--------|
| `Ctrl+F` | Start search |
| `Ctrl+N` / `Ctrl+P` | Next / previous match |
| `Ctrl+I` | Toggle case sensitivity |
| `Enter` (search) | Select current match |
| `Ctrl+R` | Find and replace |
| `Enter` (replace) | Replace current match |
| `Ctrl+A` (replace) | Replace all matches |

### Selection & Clipboard
| Key | Action |
|-----|--------|
| `Shift+↑↓←→` | Extend selection |
| `Ctrl+Shift+←/→` | Extend selection by word |
| `Ctrl+A` | Select all |
| `Ctrl+C / X / V` | Copy / Cut / Paste |

### File & Navigation
| Key | Action |
|-----|--------|
| `Ctrl+S` | Save |
| `Ctrl+Q` | Quit (prompts if unsaved) |
| `Ctrl+O` | Go back to previous file |
| `Ctrl+L` | Toggle line numbers |
| `Ctrl+P` | Command palette |
| `Ctrl+H` | Help overlay |

---

## Configuration

On first run, `chuch-term` creates `~/.config/chuch-term/config.toml` with defaults:

```toml
[editor]
line_numbers = true
relative_numbers = false
syntax_highlight = true

[clipboard]
# "auto" = detect system clipboard (default)
# "internal" = never use system clipboard (session only)
# "osc52" = force OSC-52 escape sequences (best for SSH)
strategy = "auto"
```

Open with `Ctrl+P → Open Config`. Changes are picked up within 2 seconds — no restart needed.
Legacy keys such as `editor.tab_width` and `[theme]` are tolerated but ignored.

---

## Platform Support

| Platform | Status |
|----------|--------|
| macOS (Apple Silicon / Intel) | Supported |
| Linux x86\_64 | Supported |
| Linux aarch64 (Raspberry Pi, ARM servers) | Supported |
| Any server with a shell | Supported (copy binary) |

---

## Contributing

Bug reports and pull requests are welcome.

- **Bug**: Open an issue with steps to reproduce and your OS/terminal info
- **Feature request**: Open an issue and describe the use case
- **PR**: Keep changes focused — one feature or fix per PR

Before submitting a PR:
```bash
cargo build         # must succeed
cargo test          # must pass
cargo clippy -- -D warnings   # must be clean
```

---

## License

MIT — see [LICENSE](LICENSE).

See [SECURITY.md](SECURITY.md) for vulnerability reporting.
