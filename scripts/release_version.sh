#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'EOF'
Usage:
  scripts/release_version.sh bump <version> [--lts]
  scripts/release_version.sh write-homebrew-formula <version> <macos_arm_sha> <macos_intel_sha> <linux_x86_64_sha> <output_path>
EOF
}

current_version() {
  sed -n 's/^version = "\(.*\)"/\1/p' "${REPO_ROOT}/Cargo.toml" | head -n 1
}

current_rust_version() {
  sed -n 's/^rust-version = "\(.*\)"/\1/p' "${REPO_ROOT}/Cargo.toml" | head -n 1
}

replace_in_file() {
  local path="$1"
  local old_text="$2"
  local new_text="$3"

  OLD_TEXT="$old_text" NEW_TEXT="$new_text" perl -0pi -e '
    my $old = $ENV{OLD_TEXT};
    my $new = $ENV{NEW_TEXT};
    s/\Q$old\E/$new/g;
  ' "$path"
}

ensure_changelog_section() {
  local version="$1"
  local lts_flag="$2"
  local changelog="${REPO_ROOT}/CHANGELOG.md"
  local today
  today="$(date +%F)"

  if grep -Fq "## [${version}] - ${today}" "$changelog"; then
    return
  fi

  local note=""
  if [[ "$lts_flag" == "true" ]]; then
    note=$'\n> First stable LTS release.\n'
  fi

  CHANGELOG_PATH="$changelog" RELEASE_VERSION="$version" RELEASE_DATE="$today" RELEASE_NOTE="$note" perl -0pi -e '
    my $path = $ENV{CHANGELOG_PATH};
    my $version = $ENV{RELEASE_VERSION};
    my $date = $ENV{RELEASE_DATE};
    my $note = $ENV{RELEASE_NOTE};
    my $insert = <<"BLOCK";
## [$version] - $date
$note
### Fixed
- TBD

### Changed
- TBD

### Added
- TBD

BLOCK
    s/## \[Unreleased\]\n\n/## [Unreleased]\n\n$insert/ or die "Could not insert changelog section into $path\n";
  ' "$changelog"
}

bump() {
  local new_version="$1"
  local lts_flag="${2:-false}"
  local old_version
  old_version="$(current_version)"
  local rust_version
  rust_version="$(current_rust_version)"

  replace_in_file "${REPO_ROOT}/Cargo.toml" "version = \"${old_version}\"" "version = \"${new_version}\""

  OLD_VERSION="$old_version" NEW_VERSION="$new_version" perl -0pi -e '
    my $old = $ENV{OLD_VERSION};
    my $new = $ENV{NEW_VERSION};
    s/(\[\[package\]\]\nname = "chuch-terminal"\nversion = ")\Q$old\E(")/$1$new$2/s
      or die "Could not update chuch-terminal package version in Cargo.lock\n";
  ' "${REPO_ROOT}/Cargo.lock"

  replace_in_file "${REPO_ROOT}/README.md" "version-${old_version}-" "version-${new_version}-"
  replace_in_file "${REPO_ROOT}/README.md" "before ${old_version}" "before ${new_version}"
  replace_in_file "${REPO_ROOT}/README.md" "in \`${old_version}\`" "in \`${new_version}\`"
  replace_in_file "${REPO_ROOT}/docs/install.md" "Default \`${old_version}\` behaviour:" "Default \`${new_version}\` behaviour:"
  replace_in_file "${REPO_ROOT}/docs/architecture.md" "Supported config keys in ${old_version}:" "Supported config keys in ${new_version}:"
  replace_in_file "${REPO_ROOT}/docs/architecture.md" "Render mode contract in \`${old_version}\`:" "Render mode contract in \`${new_version}\`:"
  replace_in_file "${REPO_ROOT}/docs/architecture.md" "fixed in \`${old_version}\`" "fixed in \`${new_version}\`"
  RUST_VERSION="$rust_version" perl -0pi -e '
    my $rust = $ENV{RUST_VERSION};
    s/rust-[0-9.]+\+-orange/rust-$rust+-orange/g;
    s/Rust `?[0-9.]+\+`?/Rust `$rust+`/g;
    s/Rust [0-9.]+\+/Rust $rust+/g;
    s/If you have Rust installed \([0-9.]+\+\):/If you have Rust installed ($rust+):/g;
  ' "${REPO_ROOT}/README.md" "${REPO_ROOT}/docs/install.md" "${REPO_ROOT}/SECURITY.md" "${REPO_ROOT}/docs/release_instructions.md"

  if [[ "$lts_flag" == "true" ]]; then
    replace_in_file "${REPO_ROOT}/README.md" "first stable LTS baseline is \`${old_version}\`" "first supported LTS baseline is \`${new_version}\`"
    replace_in_file "${REPO_ROOT}/SECURITY.md" "first stable LTS baseline is \`${old_version}\`" "first supported LTS baseline is \`${new_version}\`"
    replace_in_file "${REPO_ROOT}/README.md" "first supported LTS baseline is \`${old_version}\`" "first supported LTS baseline is \`${new_version}\`"
    replace_in_file "${REPO_ROOT}/SECURITY.md" "first supported LTS baseline is \`${old_version}\`" "first supported LTS baseline is \`${new_version}\`"
  fi

  ensure_changelog_section "$new_version" "$lts_flag"
}

write_homebrew_formula() {
  local version="$1"
  local macos_arm_sha="$2"
  local macos_intel_sha="$3"
  local linux_x86_64_sha="$4"
  local output_path="$5"

  cat > "$output_path" <<EOF
class ChuchTerm < Formula
  desc "Fast, minimal terminal text editor"
  homepage "https://github.com/KrzysPawlo/chuch-term"
  version "${version}"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/KrzysPawlo/chuch-term/releases/download/v${version}/chuch-term-macos-arm.tar.gz"
      sha256 "${macos_arm_sha}"
    end

    on_intel do
      url "https://github.com/KrzysPawlo/chuch-term/releases/download/v${version}/chuch-term-macos-intel.tar.gz"
      sha256 "${macos_intel_sha}"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/KrzysPawlo/chuch-term/releases/download/v${version}/chuch-term-linux-x86_64.tar.gz"
      sha256 "${linux_x86_64_sha}"
    end
  end

  def install
    bin.install "chuch-term"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/chuch-term --version")
  end
end
EOF
}

main() {
  if [[ $# -lt 1 ]]; then
    usage
    exit 1
  fi

  case "$1" in
    bump)
      if [[ $# -lt 2 || $# -gt 3 ]]; then
        usage
        exit 1
      fi
      local lts_flag="false"
      if [[ "${3:-}" == "--lts" ]]; then
        lts_flag="true"
      elif [[ $# -eq 3 ]]; then
        usage
        exit 1
      fi
      bump "$2" "$lts_flag"
      ;;
    write-homebrew-formula)
      if [[ $# -ne 6 ]]; then
        usage
        exit 1
      fi
      write_homebrew_formula "$2" "$3" "$4" "$5" "$6"
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

main "$@"
