use std::io::Write as _;

pub fn copy_to_clipboard(text: &str, strategy: &str) -> bool {
    if strategy != "internal" {
        // macOS
        if try_pbcopy(text) {
            return true;
        }
        // Linux Wayland
        if try_wl_copy(text) {
            return true;
        }
        // Linux X11
        if try_xclip(text) {
            return true;
        }
        // OSC-52
        if strategy == "osc52" || strategy == "auto" {
            osc52_copy(text);
            return true;
        }
    }
    false
}

pub fn paste_from_clipboard(strategy: &str) -> Option<String> {
    if strategy != "internal" {
        if let Some(t) = try_pbpaste() {
            return Some(t);
        }
        if let Some(t) = try_wl_paste() {
            return Some(t);
        }
        if let Some(t) = try_xclip_paste() {
            return Some(t);
        }
    }
    None
}

fn try_pbcopy(text: &str) -> bool {
    use std::process::{Command, Stdio};
    if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        return child.wait().map(|s| s.success()).unwrap_or(false);
    }
    false
}

fn try_pbpaste() -> Option<String> {
    let out = std::process::Command::new("pbpaste").output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

fn try_wl_copy(text: &str) -> bool {
    use std::process::{Command, Stdio};
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        return child.wait().map(|s| s.success()).unwrap_or(false);
    }
    false
}

fn try_wl_paste() -> Option<String> {
    let out = std::process::Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

fn try_xclip(text: &str) -> bool {
    use std::process::{Command, Stdio};
    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        return child.wait().map(|s| s.success()).unwrap_or(false);
    }
    false
}

fn try_xclip_paste() -> Option<String> {
    let out = std::process::Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

fn osc52_copy(text: &str) {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
    print!("\x1b]52;c;{encoded}\x07");
    let _ = std::io::stdout().flush();
}
