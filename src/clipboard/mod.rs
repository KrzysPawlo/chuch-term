use std::io::Write as _;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const CLIPBOARD_TIMEOUT: Duration = Duration::from_millis(250);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardCopyResult {
    System,
    Osc52,
    InternalOnly,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardPasteResult {
    System(String),
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandStatus {
    Success,
    Failure,
    Unavailable,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandOutput {
    Success(String),
    Failure,
    Unavailable,
    TimedOut,
}

pub fn copy_to_clipboard(text: &str, strategy: &str) -> ClipboardCopyResult {
    copy_with_runner(text, strategy, run_command_with_input, osc52_copy)
}

pub fn paste_from_clipboard(strategy: &str) -> ClipboardPasteResult {
    paste_with_runner(strategy, run_command_capture_stdout)
}

fn copy_with_runner<F, G>(
    text: &str,
    strategy: &str,
    run: F,
    osc52: G,
) -> ClipboardCopyResult
where
    F: Fn(&str, &[&str], &str, Duration) -> CommandStatus,
    G: Fn(&str),
{
    match strategy {
        "internal" => ClipboardCopyResult::InternalOnly,
        "osc52" => {
            osc52(text);
            ClipboardCopyResult::Osc52
        }
        _ => {
            for (program, args) in [
                ("pbcopy", &[][..]),
                ("wl-copy", &[][..]),
                ("xclip", &["-selection", "clipboard"][..]),
            ] {
                if run(program, args, text, CLIPBOARD_TIMEOUT) == CommandStatus::Success {
                    return ClipboardCopyResult::System;
                }
            }

            if strategy == "auto" {
                osc52(text);
                ClipboardCopyResult::Osc52
            } else {
                ClipboardCopyResult::Unavailable
            }
        }
    }
}

fn paste_with_runner<F>(strategy: &str, run: F) -> ClipboardPasteResult
where
    F: Fn(&str, &[&str], Duration) -> CommandOutput,
{
    match strategy {
        "internal" | "osc52" => ClipboardPasteResult::Unavailable,
        _ => {
            for (program, args) in [
                ("pbpaste", &[][..]),
                ("wl-paste", &["--no-newline"][..]),
                ("xclip", &["-selection", "clipboard", "-o"][..]),
            ] {
                if let CommandOutput::Success(text) = run(program, args, CLIPBOARD_TIMEOUT) {
                    return ClipboardPasteResult::System(text);
                }
            }
            ClipboardPasteResult::Unavailable
        }
    }
}

fn run_command_with_input(
    program: &str,
    args: &[&str],
    input: &str,
    timeout: Duration,
) -> CommandStatus {
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            return if err.kind() == std::io::ErrorKind::NotFound {
                CommandStatus::Unavailable
            } else {
                CommandStatus::Failure
            };
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(input.as_bytes()).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return CommandStatus::Failure;
        }
    }

    wait_for_status(child, timeout)
}

fn run_command_capture_stdout(program: &str, args: &[&str], timeout: Duration) -> CommandOutput {
    let mut child = match Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            return if err.kind() == std::io::ErrorKind::NotFound {
                CommandOutput::Unavailable
            } else {
                CommandOutput::Failure
            };
        }
    };

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return CommandOutput::Failure;
                }
                return child
                    .wait_with_output()
                    .map(|output| {
                        CommandOutput::Success(String::from_utf8_lossy(&output.stdout).into_owned())
                    })
                    .unwrap_or(CommandOutput::Failure);
            }
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return CommandOutput::TimedOut;
                }
                thread::sleep(POLL_INTERVAL);
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return CommandOutput::Failure;
            }
        }
    }
}

fn wait_for_status(mut child: Child, timeout: Duration) -> CommandStatus {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    CommandStatus::Success
                } else {
                    CommandStatus::Failure
                };
            }
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return CommandStatus::TimedOut;
                }
                thread::sleep(POLL_INTERVAL);
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return CommandStatus::Failure;
            }
        }
    }
}

fn osc52_copy(text: &str) {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
    print!("\x1b]52;c;{encoded}\x07");
    let _ = std::io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn internal_copy_never_calls_system_runner() {
        let called = Cell::new(false);
        let result = copy_with_runner(
            "hello",
            "internal",
            |_, _, _, _| {
                called.set(true);
                CommandStatus::Failure
            },
            |_| {},
        );

        assert_eq!(result, ClipboardCopyResult::InternalOnly);
        assert!(!called.get());
    }

    #[test]
    fn auto_copy_uses_first_available_system_backend() {
        let result = copy_with_runner(
            "hello",
            "auto",
            |program, _, _, _| {
                if program == "wl-copy" {
                    CommandStatus::Success
                } else {
                    CommandStatus::Unavailable
                }
            },
            |_| {},
        );

        assert_eq!(result, ClipboardCopyResult::System);
    }

    #[test]
    fn auto_copy_falls_back_to_osc52_when_system_clipboard_is_unavailable() {
        let osc52_calls = Cell::new(0);
        let result = copy_with_runner(
            "hello",
            "auto",
            |_, _, _, _| CommandStatus::TimedOut,
            |_| osc52_calls.set(osc52_calls.get() + 1),
        );

        assert_eq!(result, ClipboardCopyResult::Osc52);
        assert_eq!(osc52_calls.get(), 1);
    }

    #[test]
    fn osc52_strategy_skips_system_backends() {
        let called = Cell::new(false);
        let osc52_calls = Cell::new(0);
        let result = copy_with_runner(
            "hello",
            "osc52",
            |_, _, _, _| {
                called.set(true);
                CommandStatus::Success
            },
            |_| osc52_calls.set(osc52_calls.get() + 1),
        );

        assert_eq!(result, ClipboardCopyResult::Osc52);
        assert!(!called.get());
        assert_eq!(osc52_calls.get(), 1);
    }

    #[test]
    fn auto_paste_returns_first_system_output() {
        let result = paste_with_runner("auto", |program, _, _| {
            if program == "xclip" {
                CommandOutput::Success("from-xclip".to_string())
            } else {
                CommandOutput::Unavailable
            }
        });

        assert_eq!(result, ClipboardPasteResult::System("from-xclip".to_string()));
    }

    #[test]
    fn osc52_paste_is_unavailable() {
        let result = paste_with_runner("osc52", |_, _, _| {
            panic!("runner should not be called in osc52 mode");
        });

        assert_eq!(result, ClipboardPasteResult::Unavailable);
    }

    #[test]
    fn auto_paste_times_out_cleanly() {
        let result = paste_with_runner("auto", |_, _, _| CommandOutput::TimedOut);
        assert_eq!(result, ClipboardPasteResult::Unavailable);
    }
}
