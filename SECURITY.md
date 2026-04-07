# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.2.x   | ✅        |
| < 0.2   | ❌        |

## Reporting a Vulnerability

Please report security issues via GitHub Issues with the label `security`.

## Security Characteristics

`chuch-term` is a **local-only terminal text editor**:

- **No network access** — zero outbound connections, no HTTP clients, no telemetry
- **No remote data collection** — no analytics, no crash reporting, no usage tracking
- **No external API calls** — all operations are local file system reads and writes
- **No credentials stored** — the config file (`~/.config/chuch-term/config.toml`) contains only editor preferences
- **Atomic file saves** — uses a tmp → rename pattern to prevent data loss on crash
- **Clipboard integration** — uses system-provided tools (`pbcopy`, `wl-copy`, `xclip`) as subprocesses; no clipboard daemon is installed

The attack surface is limited to:
1. Reading and writing files that the user explicitly opens
2. Executing standard system clipboard utilities already present on the OS

## Dependencies

All dependencies are well-known crates from crates.io. Run `cargo audit` to verify
there are no known vulnerabilities in the dependency tree.
