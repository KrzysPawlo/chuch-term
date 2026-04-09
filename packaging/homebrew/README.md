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

Tag-driven release automation is responsible for the final formula update.
The workflow computes archive SHA256 digests from release assets and calls:

```bash
scripts/release_version.sh write-homebrew-formula <version> <arm_sha> <intel_sha> <linux_sha> <output_path>
```

That generated formula is written to:
- `packaging/homebrew/chuch-term.rb`
- `homebrew-chuch/Formula/chuch-term.rb`

Manual fallback is only for reruns or emergency repair.

Users install with:

```bash
brew tap KrzysPawlo/chuch
brew install chuch-term
```
