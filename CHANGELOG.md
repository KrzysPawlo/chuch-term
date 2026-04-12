# Changelog

All notable changes to this project will be documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

## [Unreleased]

## [0.6.7] - 2026-04-12

### Fixed
- **Managed alias reliability** — personal aliases in `~/.local/bin` are now installed as stable launchers that delegate to `chuch-term` instead of pointing at one concrete executable path, so they survive Homebrew and Linux upgrade paths much more reliably
- **Legacy alias recovery** — stale managed aliases from the older symlink-based LTS implementation are now recognized as upgradeable/removable instead of getting stuck in a conflict state

### Changed
- **Canonical config path** — `chuch-term` now uses `~/.config/chuch/config.toml` as its active config file path for the `0.6 LTS` line
- **Homebrew cleanup behavior** — first run on `0.6.7` removes the old `~/.config/chuch-term/` directory and starts from a clean config at the new path instead of attempting backward-compatible migration
- **Uninstall cleanup** — `chuch-term --uninstall` now removes both the current config directory and the legacy `~/.config/chuch-term/` directory if it still exists

## [0.6.6] - 2026-04-09

### Fixed
- **Save fidelity** — unchanged files now round-trip byte-for-byte, preserving UTF-8 BOM state, missing trailing newlines, and mixed `LF` / `CRLF` line endings instead of silently normalizing on save
- **Unicode cursor/edit correctness** — cursor movement, delete/backspace, selection boundaries, and mouse hit-testing now clamp on grapheme clusters instead of raw `char` boundaries
- **Replace-all hot path** — replace-all now applies row-aware edits with transactional undo/redo instead of rebuilding the entire document through one giant string replacement
- **Literal search churn** — incremental search now uses a literal scanner instead of compiling a fresh regex on every refresh

### Changed
- **LTS patch line** — `0.6.6` is now the current shipped patch in the `0.6 LTS` line; public branding stays `0.6 LTS` while the package version advances normally
- **Release helper scope** — `scripts/release_version.sh` now updates tracked public files only and no longer assumes ignored local `docs/` are part of the release contract
- **Large-file behavior** — editor hot paths avoid extra whole-line clones in render and keep offset lookups cached inside the buffer model

### Added
- **DevOps file support** — first-class language detection for `.log`, rotated logs, `.out`, `.err`, `.conf`, `.cfg`, `.ini`, `.env`, `.properties`, plus explicit `.txt` / `.text` recognition
- **Log and Config highlighting** — dedicated tokenizers for timestamps, levels, key-value fragments, INI sections, booleans, numeric literals, quoted values, and variable placeholders
- **Release helper command** — `scripts/release_version.sh bump-next-lts` for consistent patch bumps on the active LTS line

## [0.6.5] - 2026-04-09

> First supported LTS release.

### Fixed
- **Release-contract drift** — the project now ships on one explicit modern Rust baseline instead of claiming compatibility with an older toolchain while resolving newer `edition2024` dependencies in CI
- **Dependency-policy blocker** — the TUI stack now uses the modern `ratatui 0.30` line, removing the archived `paste` crate from the shipped dependency tree and unblocking `cargo deny`
- **Clap MSRV drift** — CLI dependencies are now pinned to the curated `4.6.0` line instead of an open-ended major that could silently jump the required toolchain
- **UI shortcut drift** — help overlay, hints bar, command palette, settings hints, and status messaging now render active shortcuts from one runtime keymap source instead of hard-coded labels
- **Config safety** — settings saves now use an atomic write+rename path, and hot reload keeps the last good runtime config instead of partially applying invalid shortcut edits
- **Overlay compact gap** — help, settings, keybindings, and command palette now fall back to truthful compact states instead of rendering blank or misleading small-terminal views
- **Alias trust surface** — managed command aliases now validate strictly, refuse unrelated path collisions, and never mutate files unless the user explicitly installs or removes the alias

### Changed
- **Toolchain contract** — `chuch-term` now targets Rust `1.94+`, package edition `2024`, and an exact CI contract job on Rust `1.94.1`
- **LTS baseline** — `0.6.5` replaces `0.6.4` as the first supported LTS release after the `0.6.4` candidate exposed final toolchain and supply-chain blockers
- **Release/docs surface** — README, install docs, security docs, release instructions, workflow copy, and version tooling now all describe the same `0.6.5` / Rust `1.94+` contract
- **Shortcut contract** — `0.6.5` now ships with a default `ctrl` profile, optional `alt` profile, and per-action override support for the curated modifier-based command set
- **CLI polish** — help and usage output now show the invoked command name, so installed personal aliases behave naturally without rebranding the product

### Added
- Dual CI guardrail: floating `stable` test coverage plus exact Rust `1.94.1` contract validation before release jobs
- Dedicated shortcut customization overlay with conflict validation and profile-default reset
- Regression coverage for shortcut resolution, invalid override rejection, profile switching, and trust-preserving config fallback
- Managed personal command alias support via `[command].alias` plus explicit install/remove actions in Settings

## [0.6.4] - 2026-04-09

> Aborted pre-LTS release candidate. `0.6.5` is the first supported LTS baseline.

### Fixed
- **UTF-8 boundary hardening** — cursor movement, buffer mutations, selection ranges, and edit actions now clamp misaligned byte offsets before slicing or mutating text, removing panic-risk from malformed runtime positions
- **Selection/edit consistency** — range reads, deletes, newline insertion, duplicate-line cursor restore, and word navigation now normalize through the same buffer position contract instead of relying on scattered local assumptions
- **Settings shortcut surface drift** — help overlay, command palette metadata, README, and local release/install docs now all document the same settings entrypoints

### Changed
- **Settings access** — `Ctrl+T` is now a second official shortcut for the settings overlay; `Alt+,` remains supported and unchanged
- **Release contract** — CI now verifies the tag matches `Cargo.toml`, validates the declared Rust contract, and prepares automated Homebrew formula sync from release assets instead of manual copy-paste
- **Version tooling** — release bumping and Homebrew formula rendering are now centralized around `scripts/release_version.sh`, with `Cargo.toml` as the canonical version source

### Added
- Regression tests for misaligned UTF-8 cursor offsets in buffer, cursor, selection, and newline paths
- Automated Homebrew sync workflow support for both the main repo formula and the `homebrew-chuch` tap repo
- User-facing `0.6.4` candidate messaging across README, security docs, and release notes

## [0.6.3] - 2026-04-08

### Fixed
- **Cross-device color reliability** — rendering no longer trusts `COLORTERM=truecolor` by itself; `chuch-term` now resolves an effective color backend before drawing any UI
- **Apple Terminal compatibility** — in the default `auto` mode, Apple Terminal now uses an ANSI-256 fallback palette instead of full RGB, preventing the magenta / washed-out color failures reproduced on older macOS + Terminal.app combinations
- **Color diagnostics truthfulness** — `chuch-term --debug-env` now separates declared signals from the effective render mode and explains why the mode was chosen
- **UI color consistency** — editor, overlays, command palette, line numbers, search/replace bars, status bar, and hints bar now render through one resolved palette instead of mixing ad-hoc RGB construction

### Added
- **`[render]` config section** with `color_mode = "auto" | "rgb" | "ansi256"`
- **ANSI-256 compatibility palette** — user theme colors and built-in design tokens are quantized through the same palette layer when RGB is not trusted
- Regression tests for:
  - terminal capability resolution
  - RGB vs ANSI-256 palette mapping
  - `--debug-env` effective-mode reporting

### Changed
- **Default color policy** — `auto` is now the recommended and default mode; `rgb` is an expert override for terminals that truly render 24-bit color correctly
- **README / install / architecture / release docs** now explain the difference between announced terminal capabilities and the render mode actually selected by `chuch-term`

## [0.6.2] - 2026-04-08

### Fixed
- **Single canonical config path on macOS and Linux** — `chuch-term` now uses only `~/.config/chuch-term/config.toml`; the accidental `~/Library/Application Support/chuch-term` path from `0.6.1` is no longer used at runtime
- **Color diagnostics alignment** — `chuch-term --debug-env` now reports the active config path, whether the config exists, and the active `theme` values (`accent`, `warning`, `dim`, `bg_bar`) in addition to terminal environment data
- **Mouse click leakage** — left-click cursor movement is now limited to the real editor area in normal editing mode; clicks on overlays, the status bar, and the bottom bars no longer move the cursor in the buffer
- **Clipboard fail-soft behavior** — system clipboard commands are now timeout-bounded and fall back cleanly instead of potentially stalling the UI on managed macOS/Linux setups
- **Docs/runtime contract drift** — README, install guide, architecture notes, release instructions, and user-facing comments now consistently document `Alt+,`, active `[theme]` support, and the canonical config lifecycle

### Changed
- **Uninstall guidance** — the recommended clean uninstall flow is now documented as:
  - `chuch-term --uninstall`
  - `brew uninstall chuch-term`
- **Color troubleshooting** — docs now explain that truecolor support is required for the intended UI and include explicit cleanup commands for the stale `0.6.1` macOS config artifact
- **Reproducible release commands** — install and release docs now prefer `--locked` cargo commands so validation and source installs use the committed dependency graph

### Added
- Regression tests for:
  - config path resolution
  - `--debug-env` reporting
  - clipboard timeout / fallback paths
  - mouse click bounds / overlay behavior
  - bottom-bar widgets honoring `theme.bg_bar`

## [0.6.1] - 2026-04-08

### Fixed
- **Settings shortcut** changed from `Ctrl+,` to `Alt+,` (`Option+,` on macOS) —
  `Ctrl+,` is outside the standard `Ctrl+A–Z` range and caused a system beep on macOS
  before the terminal could pass the event to chuch-term
- **Bottom bar colour bleed** — hints bar and status bar now explicitly set both
  foreground and background colour for every cell; previously `set_bg()` was called
  without `set_fg()`, leaving stale foreground from a previous ratatui frame which
  caused a magenta/pink bar on terminals with non-default or non-truecolor setups
- **Selection ghost** (`Ctrl+A` → `Ctrl+X`) — editor cells now use an explicit dark
  background (`#121212`) instead of `Color::Reset`; `Color::Reset` deferred to the
  terminal's own default background colour, which could leave coloured artefacts
  after selection was cleared or text was deleted
- **Command Palette wrap-around** — `↓` on the last item now jumps to the first;
  `↑` on the first item now jumps to the last
- **Command Palette contrast** — description text colour changed from `#5a5a5a` to
  `#787878` for better readability on dim displays and non-truecolor terminals
- **`[theme]` section in config.toml is now fully functional** — `accent`, `warning`,
  `dim`, and `bg_bar` accept hex colour strings (e.g. `accent = "#b0c4c8"`) and are
  applied across all UI components: hints bar, status bar, command palette, settings
  overlay, help overlay, line numbers, search/replace/goto/saveas bars, and the welcome
  screen. Changes are hot-reloaded within 2 seconds. Settings overlay close preserves
  the `[theme]` section in `config.toml`
- **Help overlay** now shows `Ctrl+D` (duplicate line) and `Alt+,` (settings overlay),
  which were missing despite both features being fully implemented since v0.6.0

### Added
- **`--debug-env` flag** — `chuch-term --debug-env` prints `TERM`, `COLORTERM`,
  `TERM_PROGRAM`, terminal size, OS/arch, and detected colour depth; useful for
  diagnosing rendering issues across different machines and terminal emulators
- **README: Requirements section** — documents truecolor terminal requirements,
  how to set `COLORTERM=truecolor`, and minimum CLT version on macOS

## [0.6.0] - 2026-04-08

### Added
- **Auto-indent** — `Enter` preserves leading whitespace of the current line; controlled by `editor.auto_indent` (default `true`)
- **Expand tabs** — `Tab` inserts spaces instead of a literal tab; `editor.expand_tabs` (default `true`), width via `editor.tab_width` (default `4`)
- **Duplicate line** — `Ctrl+D` copies the current line below and moves the cursor there; fully undoable
- **Styled cursor position** — status bar now shows `Ln X  Col Y` with accent-coloured numbers instead of plain `row:col`
- **Mouse support** — left click positions the cursor; clears selection; requires terminal to support mouse events
- **Indent guides** — optional `│` markers at every `tab_width` column in leading whitespace; `editor.indent_guides` (default `false`)
- **Indentation error hints** — red background on leading whitespace of lines with inconsistent indentation in YAML, Python, and Proto3 files; `editor.indent_errors` (default `false`); colour configurable via `editor.indent_error_bg = [r, g, b]`
- **Settings overlay** — `Ctrl+,` opens an interactive settings panel (changed to `Alt+,` in v0.6.1); `↑/↓` to navigate, `Space/Enter` to toggle, `←/→` to adjust numeric/enum values; `Esc` closes and saves all changes to `config.toml`
- `open settings` command added to the command palette

## [0.5.9] - 2026-04-08

### Added
- Save-as mode — `Ctrl+S` on a new buffer (no filename) opens a prompt at the bottom: type a path and press `Enter` to save, `Esc` to cancel; `~` expansion supported

### Fixed
- `chuch-term nonexistent.json` now opens an empty buffer pre-named `nonexistent.json` instead of exiting with "Cannot open file" — matches nano/vim behaviour

## [0.5.8] - 2026-04-08

### Changed
- Release notes and README: install instructions rewritten with explicit numbered steps, file table, and per-platform code blocks — no ambiguity about which files to download or which commands to run

## [0.5.7] - 2026-04-08

### Fixed
- SHA256 checksum is now computed on the **binary**, not the archive — verification works correctly when Safari auto-decompresses `.tar.gz` to `.tar`
- Release assets renamed: `*.tar.gz.sha256` → `*.sha256`
- README and SECURITY.md install instructions updated to reflect Safari behavior

## [0.5.6] - 2026-04-08

### Changed
- `ratatui` upgraded `0.29` → `0.30` — removes `paste` (unmaintained proc-macro) from the dependency tree entirely
- `dirs` dependency removed — replaced with 8 lines of stdlib code using `$HOME` / `$XDG_CONFIG_HOME`; eliminates `option-ext` (MPL-2.0) and `dirs-sys` from the tree
- `deny.toml`: removed advisory suppression for `RUSTSEC-2024-0436` (no longer needed) and `MPL-2.0` from license allowlist (no longer present)

## [0.5.5] - 2026-04-08

### Fixed
- `deny.toml`: added `MPL-2.0` to license allowlist (`option-ext` via `dirs`)
- `deny.toml`: suppressed `RUSTSEC-2024-0436` (`paste` unmaintained, transitive via `ratatui`, no safe upgrade)

## [0.5.4] - 2026-04-08

### Added
- `deny.toml` — license allowlist and supply-chain policy (crates.io only, semver pins required)
- CI: `cargo deny` job checks licenses and sources on every push; build gate now requires it to pass
- SHA256 checksum files (`.sha256`) generated for every release artifact — attached to GitHub Releases alongside the tarballs
- SECURITY.md: updated supported versions (0.5.x), added checksum verification instructions, dependency audit table, no-sudo install option

### Changed
- README: install instructions now include SHA256 verification step, no-sudo `~/.local/bin` alternative, and Gatekeeper explanation
- Release notes in pipeline now include SHA256 verification commands and `xattr` step for macOS

## [0.5.3] - 2026-04-08

### Changed
- README: installation section now includes full macOS Gatekeeper workaround (`xattr -d com.apple.quarantine`) with explanation, Safari `.tar` note, and "Homebrew — coming soon" placeholder
- README: Update section simplified — same steps as install, explicit note that `xattr` is required on every update

## [0.5.2] - 2026-04-08

### Added
- `chuch-term --uninstall` — removes the binary and `~/.config/chuch-term/` in one command
- README: new sections — "What gets installed", "Update", "Uninstall" with macOS Gatekeeper notes

## [0.5.1] - 2026-04-08

### Fixed
- Help overlay now shows `Ctrl+←/→` (word navigation) and `Ctrl+W / Del` (delete word) — were missing after v0.5.0
- Command palette: `open config` moved to last position; added `delete word before` and `delete word after` entries

## [0.5.0] - 2026-04-08

### Added
- Word navigation — `Ctrl+Left` / `Ctrl+Right` jump to previous / next word; works across line boundaries
- `Ctrl+Shift+Left` / `Ctrl+Shift+Right` extend the selection by word
- Delete word — `Ctrl+W` deletes the word before the cursor (like readline); `Ctrl+Delete` deletes the word after

## [0.4.4] - 2026-04-08

### Added
- Proto3 syntax highlighting — keywords (`message`, `service`, `rpc`, `enum`, `oneof`, `repeated`, `map`, `reserved`, `stream`, `import`, `package`, `option`, `syntax`, `returns`, `extend`), scalar types (`int32`, `int64`, `uint32`, `uint64`, `sint32`, `sint64`, `fixed32`, `fixed64`, `sfixed32`, `sfixed64`, `float`, `double`, `bool`, `string`, `bytes`), field numbers, string literals, line comments
- Status bar shows `Proto3` for `.proto` files

### Changed
- CI/CD consolidated from two separate workflow files into one `pipeline.yml` — CI jobs (Test, Clippy, Audit) run on every push; Build + Release jobs run only on version tags
- README: added Proto3 to the syntax highlighting feature list

### Fixed
- macOS Intel build runner changed from deprecated `macos-13` to `macos-latest` with cross-compilation — all three platform binaries now build correctly

## [0.4.3] - 2026-04-08

### Added
- GitHub Actions CI pipeline — test, clippy, audit run on every push to `main`
- Automated release workflow — pre-built binaries built and attached to GitHub Releases on every `v*.*.*` tag
- `cargo install --git` installation method documented in README
- CHANGELOG.md (this file)

### Changed
- `Cargo.toml`: added `repository`, `homepage`, `authors`, `keywords`, `categories` metadata
- README: new Installation section with pre-built binary download table and `cargo install` instructions
- `docs/install.md`: added `cargo install --git` and binary download as primary install methods

### Fixed
- Magic numbers in command palette UI (`25`, `38`) extracted to named constants `CMD_KEY_COL`, `CMD_DESC_COL`
- `prev_char_boundary` / `next_char_boundary` consolidated from `input/mod.rs` into `editor/buffer.rs` as `pub(crate)` helpers — single source of truth for UTF-8 boundary navigation

## [0.4.2] - 2026-04-08

### Fixed
- Config creation message ("Config created: ~/.config/chuch-term/config.toml") now correctly shown on first run — was silently discarded due to double `load_config()` call
- Terminal cleanup (`disable_raw_mode`, `LeaveAlternateScreen`, `show_cursor`) no longer uses `?` — all three steps now always run on exit even if one fails
- PageUp / PageDown now use actual terminal viewport height instead of hardcoded 20 lines
- Undo history capped at 10,000 entries to prevent unbounded memory growth during long editing sessions
- Duplicate `position_after` function removed from `history.rs` — now delegates to `TextBuffer::position_after`
- Config `clipboard.strategy` validated on load; unknown values fall back to `"auto"` with a visible warning in the status bar

## [0.4.1] - 2026-03-XX

### Fixed
- Dead code warnings in `buffer.rs` resolved (`cargo clippy -D warnings` gate now passes)
- Unicode test data in `search.rs` corrected (`ŻAŻÓŁĆ` → `ZAŻÓŁĆ` — uppercase of `ż` is `ź`, not `Ż`)
- Search navigation byte offset test corrected (`col: 15` → `col: 16` for second `zażółć` match)
- Paste from internal clipboard now works correctly when system clipboard returns an empty string (added `.filter(|s| !s.is_empty())` to fallback chain)
