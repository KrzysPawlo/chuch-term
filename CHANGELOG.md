# Changelog

All notable changes to this project will be documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

## [Unreleased]

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
