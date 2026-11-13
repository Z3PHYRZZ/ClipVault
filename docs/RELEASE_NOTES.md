# ClipVault v0.1.0

First public release — a calm, private Windows clipboard manager built in Rust
(edition 2024) with Iced 0.13, rusqlite (bundled SQLite + FTS5), and windows-rs 0.62.

## What's in the box

| Asset | What it is |
|---|---|
| `ClipVault-0.1.0-Setup.exe` | **Recommended.** Per-user installer (Inno Setup 6). No admin rights, Start Menu + optional desktop shortcut, optional launch-at-startup, clean uninstall that keeps your history. |
| `clipvault.exe` | Portable single-file executable. Run from any folder, no install. |

Windows 10/11 x64 only. No .NET, no VC++ redistributable, no network needed.

## Highlights

- **Clipboard history** for text, images, and file lists — captured with an
  event-driven listener (`AddClipboardFormatListener`), zero polling, ~0% idle CPU.
- **Duplicate detection** via SHA-256 + `UNIQUE(kind, hash)` upsert — repeats bump
  `use_count` instead of creating new rows.
- **Fast search** — FTS5 full-text plus content-type, date, pinned, and favorite
  filters; paged history.
- **Actions** — pin, favorite, delete, clear-all, restore-to-clipboard.
- **Global shortcuts** — configurable show/hide + `Ctrl+Shift+1..9` quick-paste.
- **Preview** — text, normalized PNG from DIB, and file metadata with missing-file state.
- **System tray** — context menu (Show / Settings / Pause / Resume / Quit), hide-to-tray.
- **Startup & singleton** — optional HKCU Run entry, named-mutex single instance.
- **Privacy** — local-only SQLite storage, pause monitoring, clear-on-exit,
  no telemetry, no network dependency.
- **Polished UI** — exact five-color pastel-rose palette, light + complementary dark
  theme, rounded cards, soft shadows, token-driven spacing, empty states.
- **No console window** — GUI-subsystem binary; diagnostics go to
  `%LOCALAPPDATA%\ClipVault\clipvault.log`.

## Install

Double-click `ClipVault-0.1.0-Setup.exe`. (SmartScreen may warn that the binary is
unsigned — click **More info > Run anyway**.)

Full docs: [docs/index.md](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/index.md)
· [Install guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/INSTALL.md)
· [User guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/USER_GUIDE.md)
· [Changelog](https://github.com/Z3PHYRZZ/ClipVault/blob/main/CHANGELOG.md)

## Verification

- `cargo check` — 0 errors / 0 warnings
- `cargo fmt --check` — clean
- `cargo clippy --all-targets -- -D warnings` — 0 warnings
- `cargo test` — **35 passed / 0 failed**
- `cargo test --release` — 35 passed / 0 failed
- Install / run / uninstall verified (0 / stays up / 0); installed exe PE subsystem 2
- No `unwrap`/`expect` in runtime paths; no blocking calls in the GUI event loop

## Known limitations

- Manual desktop smoke tests (real clipboard, tray menu) need a live Windows session.
- `CF_BITMAP` (device-dependent) is intentionally skipped; DIB/DIBV5 are preferred.
- `ignored_apps` is stored but not yet enforced in the listener (pause is enforced).
- The "show existing window" broadcast has no receiver on the running instance yet.

MIT licensed. Built by Manuel.
