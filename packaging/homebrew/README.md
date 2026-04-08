# Homebrew Packaging — chuch-term

This directory is the packaging source of truth for Homebrew support.

## Repo split

- Main repo: `chuch-term`
  - source code
  - GitHub Releases
  - docs
  - canonical Homebrew formula and update checklist
- Tap repo: `homebrew-chuch`
  - publishes `Formula/chuch-term.rb` only

## Supported Homebrew targets

- macOS Apple Silicon
- macOS Intel
- Linux x86_64

The formula uses prebuilt GitHub release archives only. It does not build from source
inside Homebrew and it does not invoke Cargo.

## Files in this directory

- `chuch-term.rb`
  - canonical formula to mirror into the tap repo
- `RELEASE_CHECKLIST.md`
  - exact update flow for future releases

## Publish flow

1. Update the formula in this directory for the new release.
2. Mirror the same content into `homebrew-chuch/Formula/chuch-term.rb`.
3. Commit and push both repos.
4. Users install with:

```bash
brew tap KrzysPawlo/chuch
brew install chuch-term
```
