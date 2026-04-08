# Homebrew Release Checklist — chuch-term

Use this after publishing a new `vX.Y.Z` release in `KrzysPawlo/chuch-term`.

## 1. Collect release facts

Confirm the new release has these archives:

- `chuch-term-macos-arm.tar.gz`
- `chuch-term-macos-intel.tar.gz`
- `chuch-term-linux-x86_64.tar.gz`

Also confirm the binary inside each archive is still named:

- `chuch-term`

## 2. Update the formula

Edit these files:

- `packaging/homebrew/chuch-term.rb`
- `../homebrew-chuch/Formula/chuch-term.rb`

Change:

- `version`
- every release `url`
- every `sha256`

## 3. Compute or verify SHA256

Preferred: compute checksums on the downloaded archives:

```bash
shasum -a 256 chuch-term-macos-arm.tar.gz
shasum -a 256 chuch-term-macos-intel.tar.gz
shasum -a 256 chuch-term-linux-x86_64.tar.gz
```

If you are using the GitHub Releases API, copy the archive digest, not the `.sha256`
sidecar file digest.

## 4. Mirror into the tap repo

Canonical source:

- `packaging/homebrew/chuch-term.rb`

Tap destination:

- `homebrew-chuch/Formula/chuch-term.rb`

Keep both files identical.

## 5. Commit and push

Main repo:

```bash
git add packaging/homebrew/chuch-term.rb packaging/homebrew/README.md packaging/homebrew/RELEASE_CHECKLIST.md README.md docs/install.md
git commit -m "docs: refresh homebrew packaging for vX.Y.Z"
git push
```

Tap repo:

```bash
git add Formula/chuch-term.rb
git commit -m "chore: bump chuch-term to vX.Y.Z"
git push
```

## 6. User upgrade path

Users upgrade with:

```bash
brew update
brew upgrade chuch-term
```

Fresh install:

```bash
brew tap KrzysPawlo/chuch
brew install chuch-term
```
