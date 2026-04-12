use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

pub const CANONICAL_COMMAND: &str = "chuch-term";
pub const ALIAS_BIN_DIR_DISPLAY: &str = "~/.local/bin";
const MANAGED_ALIAS_MARKER: &str = "# Managed by chuch-term";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasStatusKind {
    NotConfigured,
    Unsupported,
    Invalid,
    ConfiguredNotInstalled,
    Installed,
    InstalledPathMissing,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasStatus {
    pub kind: AliasStatusKind,
    pub label: String,
    pub detail: String,
}

pub fn invoked_command_name() -> String {
    std::env::args_os()
        .next()
        .and_then(|raw| {
            Path::new(&raw)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| CANONICAL_COMMAND.to_string())
}

pub fn validate_alias_name(alias: &str) -> Result<()> {
    let alias = alias.trim();
    if alias.is_empty() {
        return Ok(());
    }
    if alias == CANONICAL_COMMAND {
        bail!("Config: command.alias must not repeat the canonical command name");
    }
    if alias.starts_with('-') {
        bail!("Config: command.alias must not start with '-'");
    }
    if !alias
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-'))
    {
        bail!(
            "Config: command.alias must use only lowercase ASCII letters, digits, '_' or '-'"
        );
    }
    Ok(())
}

pub fn validate_command_section(command: &crate::config::CommandSection) -> Result<()> {
    validate_alias_name(&command.alias)
}

pub fn alias_status(command: &crate::config::CommandSection) -> AliasStatus {
    let alias = command.alias.trim();
    if alias.is_empty() {
        return AliasStatus {
            kind: AliasStatusKind::NotConfigured,
            label: "not set".to_string(),
            detail: "No personal alias configured.".to_string(),
        };
    }

    if !cfg!(unix) {
        return AliasStatus {
            kind: AliasStatusKind::Unsupported,
            label: "unsupported".to_string(),
            detail: "Managed aliases are currently supported on Unix-like systems only."
                .to_string(),
        };
    }

    if let Err(err) = validate_alias_name(alias) {
        return AliasStatus {
            kind: AliasStatusKind::Invalid,
            label: "invalid".to_string(),
            detail: err.to_string(),
        };
    }

    let Some(alias_dir) = alias_bin_dir() else {
        return AliasStatus {
            kind: AliasStatusKind::Unsupported,
            label: "unavailable".to_string(),
            detail: format!("Cannot resolve the managed alias directory {ALIAS_BIN_DIR_DISPLAY}."),
        };
    };
    let alias_path = alias_dir.join(alias);
    let current_exe = std::env::current_exe().ok();

    match alias_install_state(&alias_path, current_exe.as_deref()) {
        AliasInstallState::Missing => AliasStatus {
            kind: AliasStatusKind::ConfiguredNotInstalled,
            label: "not installed".to_string(),
            detail: format!(
                "Alias '{alias}' is configured but not installed in {}.",
                alias_dir.display()
            ),
        },
        AliasInstallState::Installed => {
            if path_contains(&alias_dir) {
                AliasStatus {
                    kind: AliasStatusKind::Installed,
                    label: "installed".to_string(),
                    detail: format!("Alias '{alias}' launches chuch-term through the managed launcher."),
                }
            } else {
                AliasStatus {
                    kind: AliasStatusKind::InstalledPathMissing,
                    label: "installed (PATH)".to_string(),
                    detail: format!(
                        "Alias '{alias}' is installed, but {} is not currently in PATH.",
                        alias_dir.display()
                    ),
                }
            }
        }
        AliasInstallState::StaleManaged => AliasStatus {
            kind: AliasStatusKind::ConfiguredNotInstalled,
            label: "needs reinstall".to_string(),
            detail: format!(
                "Alias '{alias}' uses the older managed format and should be reinstalled from Settings."
            ),
        },
        AliasInstallState::Conflict => AliasStatus {
            kind: AliasStatusKind::Conflict,
            label: "conflict".to_string(),
            detail: format!(
                "Alias path {} already exists and is not the managed chuch-term alias entry.",
                alias_path.display()
            ),
        },
        AliasInstallState::Unknown => AliasStatus {
            kind: AliasStatusKind::Conflict,
            label: "unknown".to_string(),
            detail: format!("Cannot inspect alias path {}.", alias_path.display()),
        },
    }
}

pub fn install_alias(command: &crate::config::CommandSection, current_exe: &Path) -> Result<String> {
    let alias = command.alias.trim();
    validate_alias_name(alias)?;
    if alias.is_empty() {
        bail!("Configure a command alias before installing it.");
    }
    let alias_dir = alias_bin_dir().context("Cannot determine alias directory")?;
    install_alias_at(alias, current_exe, &alias_dir)
}

pub fn remove_alias(command: &crate::config::CommandSection, current_exe: &Path) -> Result<String> {
    let alias = command.alias.trim();
    validate_alias_name(alias)?;
    if alias.is_empty() {
        bail!("No command alias is configured.");
    }
    let alias_dir = alias_bin_dir().context("Cannot determine alias directory")?;
    remove_alias_at(alias, current_exe, &alias_dir)
}

pub fn cleanup_uninstall_alias(command: &crate::config::CommandSection, current_exe: &Path) -> Result<Option<String>> {
    let alias = command.alias.trim();
    validate_alias_name(alias)?;
    if alias.is_empty() {
        return Ok(None);
    }
    let Some(alias_dir) = alias_bin_dir() else {
        return Ok(None);
    };
    let alias_path = alias_dir.join(alias);
    if !is_managed_alias_target(&alias_path, current_exe)? {
        return Ok(None);
    }
    std::fs::remove_file(&alias_path)
        .with_context(|| format!("Cannot remove managed alias {}", alias_path.display()))?;
    Ok(Some(format!("Removed alias: {}", alias_path.display())))
}

pub(crate) fn alias_bin_dir_from_home(home: Option<OsString>) -> Option<PathBuf> {
    home.map(|home| PathBuf::from(home).join(".local").join("bin"))
}

pub(crate) fn alias_bin_dir() -> Option<PathBuf> {
    alias_bin_dir_from_home(std::env::var_os("HOME"))
}

pub(crate) fn install_alias_at(alias: &str, current_exe: &Path, alias_dir: &Path) -> Result<String> {
    validate_alias_name(alias)?;
    std::fs::create_dir_all(alias_dir)
        .with_context(|| format!("Cannot create alias directory {}", alias_dir.display()))?;
    let alias_path = alias_dir.join(alias);

    match alias_install_state(&alias_path, Some(current_exe)) {
        AliasInstallState::Installed => {
            return Ok(format!("Alias '{alias}' is already installed."));
        }
        AliasInstallState::StaleManaged => {
            std::fs::remove_file(&alias_path)
                .with_context(|| format!("Cannot replace managed alias {}", alias_path.display()))?;
        }
        AliasInstallState::Conflict => {
            bail!(
                "Alias path {} already exists and is not a managed chuch-term alias entry.",
                alias_path.display()
            );
        }
        AliasInstallState::Unknown => {
            bail!("Cannot inspect alias path {}.", alias_path.display());
        }
        AliasInstallState::Missing => {}
    }

    #[cfg(unix)]
    {
        std::fs::write(&alias_path, managed_alias_script().as_bytes())
            .with_context(|| format!("Cannot write alias launcher {}", alias_path.display()))?;
        let mut perms = std::fs::metadata(&alias_path)
            .with_context(|| format!("Cannot inspect alias launcher {}", alias_path.display()))?
            .permissions();
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        std::fs::set_permissions(&alias_path, perms)
            .with_context(|| format!("Cannot mark alias launcher executable {}", alias_path.display()))?;
        if path_contains(alias_dir) {
            Ok(format!(
                "Installed alias '{alias}' at {}. If the current shell does not see it yet, reopen the shell or run 'hash -r'.",
                alias_path.display()
            ))
        } else {
            Ok(format!(
                "Installed alias '{alias}' at {}. Add {} to PATH in ~/.zshrc or ~/.bashrc, then reopen the shell.",
                alias_path.display(),
                alias_dir.display()
            ))
        }
    }

    #[cfg(not(unix))]
    {
        bail!("Managed aliases are supported on Unix-like systems only.")
    }
}

pub(crate) fn remove_alias_at(alias: &str, current_exe: &Path, alias_dir: &Path) -> Result<String> {
    validate_alias_name(alias)?;
    let alias_path = alias_dir.join(alias);
    match alias_install_state(&alias_path, Some(current_exe)) {
        AliasInstallState::Missing => Ok(format!("Alias '{alias}' is not installed.")),
        AliasInstallState::Installed | AliasInstallState::StaleManaged => {
            std::fs::remove_file(&alias_path)
                .with_context(|| format!("Cannot remove alias {}", alias_path.display()))?;
            Ok(format!("Removed alias '{alias}' from {}.", alias_path.display()))
        }
        AliasInstallState::Conflict => bail!(
            "Refusing to remove {} because it is not a managed chuch-term alias entry.",
            alias_path.display()
        ),
        AliasInstallState::Unknown => bail!("Cannot inspect alias path {}.", alias_path.display()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AliasInstallState {
    Missing,
    Installed,
    StaleManaged,
    Conflict,
    Unknown,
}

fn alias_install_state(alias_path: &Path, current_exe: Option<&Path>) -> AliasInstallState {
    if !alias_path.exists() && std::fs::symlink_metadata(alias_path).is_err() {
        return AliasInstallState::Missing;
    }

    match inspect_alias_path(alias_path, current_exe) {
        Ok(AliasPathKind::Missing) => AliasInstallState::Missing,
        Ok(AliasPathKind::ManagedWrapper | AliasPathKind::LegacyManagedSymlinkCurrent) => {
            AliasInstallState::Installed
        }
        Ok(AliasPathKind::LegacyManagedSymlinkStale) => AliasInstallState::StaleManaged,
        Ok(AliasPathKind::Conflict) => AliasInstallState::Conflict,
        Err(_) => AliasInstallState::Unknown,
    }
}

fn is_managed_alias_target(alias_path: &Path, current_exe: &Path) -> Result<bool> {
    Ok(matches!(
        inspect_alias_path(alias_path, Some(current_exe))?,
        AliasPathKind::ManagedWrapper
            | AliasPathKind::LegacyManagedSymlinkCurrent
            | AliasPathKind::LegacyManagedSymlinkStale
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AliasPathKind {
    Missing,
    ManagedWrapper,
    LegacyManagedSymlinkCurrent,
    LegacyManagedSymlinkStale,
    Conflict,
}

fn inspect_alias_path(alias_path: &Path, current_exe: Option<&Path>) -> Result<AliasPathKind> {
    let metadata = match std::fs::symlink_metadata(alias_path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(AliasPathKind::Missing),
        Err(err) => return Err(err).with_context(|| format!("Cannot stat {}", alias_path.display())),
    };

    if metadata.file_type().is_symlink() {
        return inspect_symlink_alias(alias_path, current_exe);
    }

    if metadata.is_file() && is_managed_alias_wrapper(alias_path)? {
        return Ok(AliasPathKind::ManagedWrapper);
    }

    Ok(AliasPathKind::Conflict)
}

fn inspect_symlink_alias(alias_path: &Path, current_exe: Option<&Path>) -> Result<AliasPathKind> {
    let link_target = std::fs::read_link(alias_path)
        .with_context(|| format!("Cannot read alias symlink {}", alias_path.display()))?;
    let target_name = link_target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    if target_name != CANONICAL_COMMAND {
        return Ok(AliasPathKind::Conflict);
    }

    match current_exe {
        Some(current_exe) => {
            let resolved_target = resolve_symlink_target(alias_path, &link_target);
            let current_exe = canonicalize_if_exists(current_exe);
            if let (Some(resolved_target), Some(current_exe)) = (resolved_target, current_exe) {
                if resolved_target == current_exe {
                    Ok(AliasPathKind::LegacyManagedSymlinkCurrent)
                } else {
                    Ok(AliasPathKind::LegacyManagedSymlinkStale)
                }
            } else {
                Ok(AliasPathKind::LegacyManagedSymlinkStale)
            }
        }
        None => Ok(AliasPathKind::LegacyManagedSymlinkStale),
    }
}

fn resolve_symlink_target(alias_path: &Path, link_target: &Path) -> Option<PathBuf> {
    let resolved_target = if link_target.is_absolute() {
        link_target.to_path_buf()
    } else {
        alias_path
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .join(link_target)
    };
    canonicalize_if_exists(&resolved_target)
}

fn canonicalize_if_exists(path: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok()
}

fn managed_alias_script() -> String {
    format!(
        "#!/bin/sh\n{MANAGED_ALIAS_MARKER}\nexec {CANONICAL_COMMAND} \"$@\"\n"
    )
}

fn is_managed_alias_wrapper(alias_path: &Path) -> Result<bool> {
    let content = std::fs::read_to_string(alias_path)
        .with_context(|| format!("Cannot read alias launcher {}", alias_path.display()))?;
    Ok(content == managed_alias_script())
}

fn path_contains(dir: &Path) -> bool {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).any(|entry| entry == dir))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("chuch-term-alias-{name}-{nanos}"))
    }

    #[test]
    fn validates_alias_names() {
        assert!(validate_alias_name("cct").is_ok());
        assert!(validate_alias_name("dev_123").is_ok());
        assert!(validate_alias_name("chuch-term").is_err());
        assert!(validate_alias_name("dupa plik").is_err());
        assert!(validate_alias_name("plik.txt").is_err());
        assert!(validate_alias_name("-oops").is_err());
    }

    #[test]
    fn installs_and_removes_managed_alias_launcher() {
        let root = temp_root("install");
        let alias_dir = root.join("bin");
        let exe = root.join("chuch-term");
        std::fs::create_dir_all(&root).expect("root dir");
        std::fs::write(&exe, "binary").expect("fake binary");

        let install = install_alias_at("cct", &exe, &alias_dir).expect("install");
        assert!(install.contains("Installed alias 'cct'"));
        assert!(alias_dir.join("cct").exists());
        let script = std::fs::read_to_string(alias_dir.join("cct")).expect("alias script");
        assert_eq!(script, managed_alias_script());

        let remove = remove_alias_at("cct", &exe, &alias_dir).expect("remove");
        assert!(remove.contains("Removed alias 'cct'"));
        assert!(!alias_dir.join("cct").exists());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_to_overwrite_unrelated_alias_path() {
        let root = temp_root("conflict");
        let alias_dir = root.join("bin");
        let exe = root.join("chuch-term");
        std::fs::create_dir_all(&alias_dir).expect("alias dir");
        std::fs::write(&exe, "binary").expect("fake binary");
        std::fs::write(alias_dir.join("cct"), "not a symlink").expect("conflict file");

        let err = install_alias_at("cct", &exe, &alias_dir).expect_err("conflict expected");
        assert!(err
            .to_string()
            .contains("already exists and is not a managed"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn detects_alias_bin_dir_from_home() {
        let dir = alias_bin_dir_from_home(Some(OsString::from("/tmp/home"))).expect("alias dir");
        assert_eq!(dir, PathBuf::from("/tmp/home/.local/bin"));
    }

    #[test]
    fn refreshes_stale_legacy_symlink_alias_on_install() {
        let root = temp_root("refresh");
        let alias_dir = root.join("bin");
        let cellar_old = root.join("Cellar").join("0.6.6").join("bin");
        let cellar_new = root.join("Cellar").join("0.6.7").join("bin");
        let old_exe = cellar_old.join(CANONICAL_COMMAND);
        let new_exe = cellar_new.join(CANONICAL_COMMAND);
        std::fs::create_dir_all(&alias_dir).expect("alias dir");
        std::fs::create_dir_all(&cellar_old).expect("old cellar");
        std::fs::create_dir_all(&cellar_new).expect("new cellar");
        std::fs::write(&old_exe, "old").expect("old exe");
        std::fs::write(&new_exe, "new").expect("new exe");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&old_exe, alias_dir.join("cct")).expect("legacy alias");

        let install = install_alias_at("cct", &new_exe, &alias_dir).expect("install");
        assert!(install.contains("Installed alias 'cct'"));
        let script = std::fs::read_to_string(alias_dir.join("cct")).expect("alias script");
        assert_eq!(script, managed_alias_script());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn removes_stale_legacy_symlink_alias() {
        let root = temp_root("remove-stale");
        let alias_dir = root.join("bin");
        let cellar_old = root.join("Cellar").join("0.6.6").join("bin");
        let cellar_new = root.join("Cellar").join("0.6.7").join("bin");
        let old_exe = cellar_old.join(CANONICAL_COMMAND);
        let new_exe = cellar_new.join(CANONICAL_COMMAND);
        std::fs::create_dir_all(&alias_dir).expect("alias dir");
        std::fs::create_dir_all(&cellar_old).expect("old cellar");
        std::fs::create_dir_all(&cellar_new).expect("new cellar");
        std::fs::write(&old_exe, "old").expect("old exe");
        std::fs::write(&new_exe, "new").expect("new exe");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&old_exe, alias_dir.join("cct")).expect("legacy alias");

        let remove = remove_alias_at("cct", &new_exe, &alias_dir).expect("remove");
        assert!(remove.contains("Removed alias 'cct'"));
        assert!(!alias_dir.join("cct").exists());

        let _ = std::fs::remove_dir_all(root);
    }
}
