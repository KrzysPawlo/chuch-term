# chuch-term

![version](https://img.shields.io/badge/version-0.6.5-b0c4c8)
![license](https://img.shields.io/badge/license-MIT-green)
![rust](https://img.shields.io/badge/rust-1.94+-orange)

A minimal terminal editor for developers.

`0.6.5` is the first supported LTS baseline: modern Rust/tooling, hardened UTF-8 and config flows, configurable shortcut profiles, and a cleaner release path.

```bash
chuch-term file.rs
cct file.rs
```

`chuch-term` is the canonical command name. `cct` is an example of an optional personal alias you can configure and install from Settings.

---

## What It Is

`chuch-term` is for fast, everyday editing in a terminal when `nano` feels too limited and a full modal editor is more than you want.

It stays intentionally small:

- direct typing, no modal editing
- one binary, no plugin system
- discoverable shortcuts that match the active keymap
- enough daily-driver features for source files, config files, scripts, and quick fixes on remote boxes

---

## Why Use It

- **Fast to open** for a single file or an empty scratch buffer
- **Predictable** under resize, config reloads, and invalid input
- **Discoverable** because help, hints, palette, and settings reflect the real active shortcuts
- **Customizable** without turning into a framework
- **Portable** as a single binary on macOS and Linux

---

## What You Get

- syntax highlighting for Rust, Python, JavaScript/TypeScript, Go, TOML, YAML, Shell, Markdown, and Proto3
- incremental search and replace
- unlimited undo / redo
- selection plus system clipboard support
- line numbers, relative numbers, go-to-line, duplicate line
- shortcut profiles (`ctrl` and `alt`) with per-action overrides
- live settings overlay and dedicated shortcut editor
- managed personal command alias in `~/.local/bin`
- atomic file saves and atomic config writes
- valid-only config hot reloads

---

## First Run

Open a file:

```bash
chuch-term src/main.rs
```

Start an empty buffer:

```bash
chuch-term
```

If you prefer a shorter launch command, set a personal alias such as `cct` in Settings and install it:

```bash
cct notes.txt
```

---

## Core Workflow

- type to edit immediately
- use the active save shortcut to write the file
- use the active search shortcut to search
- use the active palette shortcut to open commands
- use the active settings shortcut to change editor options, shortcut profile, or command alias

The editor is not modal. The current keymap is always the source of truth, and the UI reflects it.

---

## Customization

### Shortcut profiles

`0.6.5` starts with the `ctrl` profile by default.

You can:

- switch to the `alt` profile
- override selected actions
- edit shortcuts from `Settings -> Customize shortcuts`

Help, hints, command palette, and settings all render the active bindings from the same runtime keymap.

### Personal command alias

You can configure one optional personal alias:

```toml
[command]
alias = "cct"
```

Important rules:

- `chuch-term` stays the canonical package and binary name
- the alias is additive, not a rename
- the app never installs or removes the alias automatically just because config changed
- alias install/remove is explicit from Settings
- managed aliases are installed only into `~/.local/bin`

Valid alias names use lowercase ASCII letters, digits, `_`, and `-`.

---

## Installation

### Homebrew

```bash
brew tap KrzysPawlo/chuch
brew install chuch-term
```

### cargo install

```bash
cargo install --git https://github.com/KrzysPawlo/chuch-term --locked
```

Get Rust from [rustup.rs](https://rustup.rs).

### Pre-built binaries

Download from [GitHub Releases](https://github.com/KrzysPawlo/chuch-term/releases/latest):

- `chuch-term-macos-arm.tar.gz`
- `chuch-term-macos-intel.tar.gz`
- `chuch-term-linux-x86_64.tar.gz`

Example for macOS Apple Silicon:

```bash
cd ~/Downloads
tar xf chuch-term-macos-arm.tar
shasum -a 256 -c chuch-term-macos-arm.sha256
sudo mv chuch-term /usr/local/bin/
xattr -d com.apple.quarantine /usr/local/bin/chuch-term
chuch-term --version
```

If you want to avoid `sudo`, install into `~/.local/bin` instead.

### Build from source

```bash
cargo build --locked --release
cp target/release/chuch-term /usr/local/bin/
```

---

## Update

Homebrew:

```bash
brew update
brew upgrade chuch-term
```

Release assets:

```bash
tar xf chuch-term-macos-arm.tar
shasum -a 256 -c chuch-term-macos-arm.sha256
sudo mv chuch-term /usr/local/bin/
xattr -d com.apple.quarantine /usr/local/bin/chuch-term
chuch-term --version
```

---

## Uninstall

Homebrew install:

```bash
brew uninstall chuch-term
```

Full uninstall:

```bash
chuch-term --uninstall
brew uninstall chuch-term
```

`--uninstall` removes:

- the current binary
- the canonical config directory `~/.config/chuch-term/`
- the managed personal alias symlink if it exists and points to the current binary

It does not delete unrelated files or arbitrary shell aliases.

---

## Configuration

On first run, `chuch-term` creates `~/.config/chuch-term/config.toml`:

```toml
[editor]
line_numbers = true
relative_numbers = false
syntax_highlight = true
auto_indent = true
expand_tabs = true
tab_width = 4
indent_guides = false
indent_errors = false

[clipboard]
strategy = "auto"

[shortcuts]
profile = "ctrl"

[shortcuts.overrides]
# settings = "comma"
# help = "b"

[command]
alias = ""

[render]
color_mode = "auto"

[theme]
accent  = "#b0c4c8"
warning = "#ff9944"
dim     = "#5a5a5a"
bg_bar  = "#121212"
```

Config behavior:

- writes are atomic
- hot reload happens within about 2 seconds
- invalid config does not replace the current runtime state
- invalid shortcut or alias edits are rejected with a warning

Open the config from the command palette or use Settings.

---

## Keybindings

The exact bindings depend on the active profile and overrides. The shipped defaults are:

- active profile + `S` save
- active profile + `Q` quit
- active profile + `F` search
- active profile + `R` replace
- active profile + `P` or `K` command palette, depending on profile
- active profile + `H` help
- active profile + `T` or `,` for settings, depending on profile

Use the built-in help overlay for the current truth.

---

## Terminal Rendering

`chuch-term` supports:

- `rgb` for known-good truecolor terminals
- `ansi256` for maximum compatibility
- `auto` as the default

In `auto`, the editor chooses the effective mode from the detected terminal instead of trusting `COLORTERM` alone.

Diagnostics:

```bash
chuch-term --debug-env
```

Recommended default:

```toml
[render]
color_mode = "auto"
```

---

## Requirements

- Rust `1.94+` to build from source
- macOS or Linux
- a normal interactive terminal

Managed command aliases in this LTS pass are supported on Unix-like systems only.

---

## Platform Support

- macOS Apple Silicon / Intel
- Linux x86_64
- Linux aarch64

---

## Contributing

Bug reports and focused pull requests are welcome.

Before submitting a PR:

```bash
cargo build --locked
cargo test --locked
cargo clippy --locked -- -D warnings
```

---

## License

MIT — see [LICENSE](LICENSE).

See [SECURITY.md](SECURITY.md) for vulnerability reporting.
