# chuch-term

![version](https://img.shields.io/badge/version-0.5.9-b0c4c8)
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

### Homebrew (recommended)

```bash
brew tap KrzysPawlo/chuch
brew install chuch-term
```

### Pre-built binary

Download from [GitHub Releases](https://github.com/KrzysPawlo/chuch-term/releases/latest):

- macOS Apple Silicon: `chuch-term-macos-arm.tar.gz`
- macOS Intel: `chuch-term-macos-intel.tar.gz`
- Linux x86_64 (static): `chuch-term-linux-x86_64.tar.gz`

**1. Download both files** from [GitHub Releases](https://github.com/KrzysPawlo/chuch-term/releases/latest):

- macOS Apple Silicon: `chuch-term-macos-arm.tar.gz` + `chuch-term-macos-arm.sha256`
- macOS Intel: `chuch-term-macos-intel.tar.gz` + `chuch-term-macos-intel.sha256`
- Linux x86_64: `chuch-term-linux-x86_64.tar.gz` + `chuch-term-linux-x86_64.sha256`

**2. Open Terminal and run** (example for macOS Apple Silicon):

```bash
cd ~/Downloads
tar xf chuch-term-macos-arm.tar                         # Safari saves as .tar — this always works
shasum -a 256 -c chuch-term-macos-arm.sha256            # should print: chuch-term: OK
sudo mv chuch-term /usr/local/bin/
xattr -d com.apple.quarantine /usr/local/bin/chuch-term # required on macOS for every downloaded binary
chuch-term --version
```

> `xattr` removes the macOS Gatekeeper quarantine flag. It is required every time you
> install a new binary downloaded from the internet and is not a security bypass —
> it is the standard way to trust a binary you have explicitly chosen to install.
> To avoid it entirely, use `cargo install --git` instead.

**No-sudo alternative** — install to your user directory (no admin password needed):

```bash
mkdir -p ~/.local/bin
mv chuch-term ~/.local/bin/
xattr -d com.apple.quarantine ~/.local/bin/chuch-term   # macOS only
# Make sure ~/.local/bin is in your PATH — add to ~/.zshrc if needed:
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

- Binary:
  - Homebrew prefix bin when installed with Homebrew
  - `/usr/local/bin/chuch-term` or `~/.local/bin/chuch-term` for manual installs
- Config:
  - `~/.config/chuch-term/config.toml` created automatically on first run

Nothing else — no background services, no shell hooks, no system-level changes.

---

## Update

If you installed with Homebrew:

```bash
brew update
brew upgrade chuch-term
```

If you installed from release assets, download the new release from [GitHub Releases](https://github.com/KrzysPawlo/chuch-term/releases/latest), then run the same steps as installation:

```bash
tar xf chuch-term-macos-arm.tar                        # adjust for your platform
shasum -a 256 -c chuch-term-macos-arm.sha256           # verify binary
sudo mv chuch-term /usr/local/bin/
xattr -d com.apple.quarantine /usr/local/bin/chuch-term
chuch-term --version
```

The `xattr` step is required on every update — macOS re-quarantines each new download.

---

## Uninstall

Homebrew install:

```bash
brew uninstall chuch-term
```

Manual install:

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

- `↑ ↓ ← →` move cursor
- `Home` / `End` jump to start or end of line
- `PgUp` / `PgDn` scroll by page
- `Ctrl+←` / `Ctrl+→` jump by word
- `Ctrl+G` go to line

### Editing

- type to insert text
- `Backspace` / `Delete` delete character
- `Enter` insert new line
- `Ctrl+Z` undo
- `Ctrl+Y` redo
- `Ctrl+W` delete word before cursor
- `Ctrl+Delete` delete word after cursor
- `Alt+U` uppercase selection
- `Alt+L` lowercase selection

### Find & Replace

- `Ctrl+F` start search
- `Ctrl+N` / `Ctrl+P` next or previous match
- `Ctrl+I` toggle case sensitivity
- `Enter` in search selects current match
- `Ctrl+R` open find and replace
- `Enter` in replace confirms current replacement
- `Ctrl+A` in replace mode replaces all matches

### Selection & Clipboard

- `Shift+↑↓←→` extend selection
- `Ctrl+Shift+←/→` extend selection by word
- `Ctrl+A` select all
- `Ctrl+C / X / V` copy, cut, paste

### File & Navigation

- `Ctrl+S` save
- `Ctrl+Q` quit, with prompt if unsaved
- `Ctrl+O` go back to previous file
- `Ctrl+L` toggle line numbers
- `Ctrl+P` command palette
- `Ctrl+H` help overlay

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

- macOS (Apple Silicon / Intel): supported
- Linux x86_64: supported
- Linux aarch64 (Raspberry Pi, ARM servers): supported
- Any server with a shell: supported via copied binary

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
