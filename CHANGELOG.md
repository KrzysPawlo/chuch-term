# Changelog

All notable changes to this project will be documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

## [Unreleased]

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
