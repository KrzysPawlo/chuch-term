use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

pub const CANONICAL_COMMAND: &str = "chuch-term";
pub const ALIAS_BIN_DIR_DISPLAY: &str = "~/.local/bin";

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
                    detail: format!(
                        "Alias '{alias}' points to the current chuch-term binary."
                    ),
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
        AliasInstallState::Conflict => AliasStatus {
            kind: AliasStatusKind::Conflict,
            label: "conflict".to_string(),
            detail: format!(
                "Alias path {} already exists and is not the managed symlink for the current binary.",
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
        AliasInstallState::Conflict => {
            bail!(
                "Alias path {} already exists and is not managed by this chuch-term binary.",
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
        std::os::unix::fs::symlink(current_exe, &alias_path)
            .with_context(|| format!("Cannot create alias symlink {}", alias_path.display()))?;
        Ok(format!(
            "Installed alias '{alias}' at {}.",
            alias_path.display()
        ))
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
        AliasInstallState::Installed => {
            std::fs::remove_file(&alias_path)
                .with_context(|| format!("Cannot remove alias {}", alias_path.display()))?;
            Ok(format!("Removed alias '{alias}' from {}.", alias_path.display()))
        }
        AliasInstallState::Conflict => bail!(
            "Refusing to remove {} because it is not the managed alias for the current binary.",
            alias_path.display()
        ),
        AliasInstallState::Unknown => bail!("Cannot inspect alias path {}.", alias_path.display()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AliasInstallState {
    Missing,
    Installed,
    Conflict,
    Unknown,
}

fn alias_install_state(alias_path: &Path, current_exe: Option<&Path>) -> AliasInstallState {
    if !alias_path.exists() && std::fs::symlink_metadata(alias_path).is_err() {
        return AliasInstallState::Missing;
    }

    match current_exe {
        Some(current_exe) => match is_managed_alias_target(alias_path, current_exe) {
            Ok(true) => AliasInstallState::Installed,
            Ok(false) => AliasInstallState::Conflict,
            Err(_) => AliasInstallState::Unknown,
        },
        None => AliasInstallState::Unknown,
    }
}

fn is_managed_alias_target(alias_path: &Path, current_exe: &Path) -> Result<bool> {
    let metadata = match std::fs::symlink_metadata(alias_path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err).with_context(|| format!("Cannot stat {}", alias_path.display())),
    };
    if !metadata.file_type().is_symlink() {
        return Ok(false);
    }

    let link_target = std::fs::read_link(alias_path)
        .with_context(|| format!("Cannot read alias symlink {}", alias_path.display()))?;
    let resolved_target = if link_target.is_absolute() {
        link_target
    } else {
        alias_path
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .join(link_target)
    };
    let resolved_target = std::fs::canonicalize(&resolved_target)
        .with_context(|| format!("Cannot resolve alias target {}", alias_path.display()))?;
    let current_exe = std::fs::canonicalize(current_exe)
        .with_context(|| format!("Cannot resolve current executable {}", current_exe.display()))?;
    Ok(resolved_target == current_exe)
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
    fn installs_and_removes_managed_alias_symlink() {
        let root = temp_root("install");
        let alias_dir = root.join("bin");
        let exe = root.join("chuch-term");
        std::fs::create_dir_all(&root).expect("root dir");
        std::fs::write(&exe, "binary").expect("fake binary");

        let install = install_alias_at("cct", &exe, &alias_dir).expect("install");
        assert!(install.contains("Installed alias 'cct'"));
        assert!(alias_dir.join("cct").exists());

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
            .contains("already exists and is not managed"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn detects_alias_bin_dir_from_home() {
        let dir = alias_bin_dir_from_home(Some(OsString::from("/tmp/home"))).expect("alias dir");
        assert_eq!(dir, PathBuf::from("/tmp/home/.local/bin"));
    }
}
