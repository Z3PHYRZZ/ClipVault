# ClipVault

> A calm, private Windows clipboard manager.

[![Release](https://img.shields.io/github/v/release/Z3PHYRZZ/ClipVault)](https://github.com/Z3PHYRZZ/ClipVault/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11-blue.svg)](https://www.microsoft.com/windows)

A calm, private Windows clipboard manager. Text, images, and file lists land in a
local SQLite history; search, pin, favorite, restore, and global quick-paste keep
them at hand. **No network, no telemetry, no accounts, no console window.** Built
with Rust (edition 2024), Iced 0.13, rusqlite (bundled SQLite + FTS5), and windows-rs.

**Download:** [latest Release](https://github.com/Z3PHYRZZ/ClipVault/releases/latest) —
grab `ClipVault-0.1.1-Setup.exe` and double-click it. Full docs in [`docs/`](docs/index.md).

| | |
|---|---|
| [Install guide](docs/INSTALL.md) | [User guide](docs/USER_GUIDE.md) |
| [Architecture](ARCHITECTURE.md) | [Changelog](CHANGELOG.md) |
| [Contributing](CONTRIBUTING.md) | [Packaging](PACKAGING.md) |

## Features

- **Clipboard history** — text (`CF_UNICODETEXT`), images (`CF_DIB`/`CF_DIBV5`),
  and file drop lists (`CF_HDROP`), captured from an event-driven listener. No polling.
- **Duplicate detection** — SHA-256 content hashes with a `UNIQUE(kind, hash)` index;
  repeats bump `use_count` instead of creating new rows (race-safe `ON CONFLICT` upsert).
- **Search & filters** — FTS5 full-text plus type / date / pinned / favorite filters,
  paged history. LIKE fallback for substrings and CJK.
- **Actions** — pin, favorite, delete, clear-all (confirm dialog), restore-to-clipboard.
- **Global shortcuts** — configurable show/hide combo plus `Ctrl+Shift+1..9` quick-paste.
- **Persistence** — versioned SQLite schema (migrations v1→v3), WAL, transactions,
  configurable retention pruning, image assets in an app-managed directory.
- **Preview** — text, image (PNG normalized from DIB), and file metadata (size, exists).
- **System tray** — Shell_NotifyIconW with a context menu, hide-to-tray, pause monitoring.
- **Startup** — optional HKCU Run registration, named-mutex single instance,
  start minimized, graceful shutdown with WAL checkpoint.
- **Privacy** — pause monitoring, clear-on-exit, ignored apps, local-only storage.
- **Polished GUI** — exact five-color coolors palette, light + complementary
  dark themes, rounded cards, soft borders, subtle shadows, empty states.

## Brand palette (v0.1.1 — official, no other accent colors)

From [coolors.co/22223b-4a4e69-9a8c98-c9ada7-f2e9e4](https://coolors.co/palette/22223b-4a4e69-9a8c98-c9ada7-f2e9e4).

| Role | Hex | Use |
|---|---|---|
| Cream | `#F2E9E4` | light-mode background, cards, light text on dark |
| Rose | `#C9ADA7` | secondary panels, buttons, active state, selection |
| Mauve | `#9A8C98` | secondary text, icons, borders, quiet buttons |
| Slate | `#4A4E69` | dark surfaces, header accents |
| Ink | `#22223B` | dark-mode background, primary text, outlines |

Light: Cream bg, Rose panels, Cream cards, Rose interactive, Ink type.
Dark: Ink bg, Slate surfaces, Cream/Rose accents, Cream text on dark —
complementary, not inverted. Every UI color, the tray icon, and the app icon are
drawn from these five hexes (alpha variants for hairlines/shadows only). Neutral
white/charcoal/grayscale permitted only for readability.

## Install (recommended)

Double-click **`installer\Output\ClipVault-0.1.1-Setup.exe`**. Per-user install
(no admin rights), Start Menu + optional desktop shortcut, optional launch at
startup, clean uninstall that keeps your clipboard history. Full installer docs
in `PACKAGING.md`. Rebuild the installer with `installer\build-installer.bat`.

## Build & run (Windows 10/11)

Requirements:
- Rust stable ≥ 1.85 (edition 2024). This machine: `rustc 1.99.1`, `x86_64-pc-windows-msvc`.
- MSVC build tools + Windows SDK (VS 18 Community, SDK `10.0.26100.0`).
- No `vcvars` needed if the MSVC toolchain is the rustup default; `cargo` finds `link.exe`.

```bash
export PATH="/c/Users/Dodot/.cargo/bin:$PATH"   # Git Bash; adjust for your install
cd "D:/job done/ClipVault"
cargo build --release
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

Run: `target\release\clipvault.exe`. On first launch the app writes
`%LOCALAPPDATA%\ClipVault\` (settings.json, clipvault.db, assets/, log).

### Data locations
- Settings: `%LOCALAPPDATA%\ClipVault\settings.json` (atomic write via temp + rename).
- Database: `%LOCALAPPDATA%\ClipVault\clipvault.db` (WAL).
- Image assets: `%LOCALAPPDATA%\ClipVault\assets\<shard>\<hash>.png`.
- Startup: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value `ClipVault`.

## Architecture

See `ARCHITECTURE.md` (design + Windows API limitations), `ROADMAP.md` (11 phases),
`TASKS.md`, `CONTRACT.md` / `FLEET.md` (frozen interfaces), and `SUMMARY.md`.

- Message-driven Iced: one `Message` enum; `update()` never blocks. DB, clipboard,
  hotkey, and tray work live on dedicated threads; a channel `Subscription` feeds
  `WorkerEvent`s back into the GUI.
- Platform code is isolated in `src/platform/` behind traits, `#[cfg(windows)]`-gated
  with headless stubs so `cargo test` runs without a desktop session.
- Windows clipboard specifics (HGLOBAL ownership transfer, bounded `OpenClipboard`
  retries, `AddClipboardFormatListener`, `GetClipboardSequenceNumber` self-filter,
  `CF_DIB` vs `CF_BITMAP`, DROPFILES layout) are documented in `ARCHITECTURE.md`
  and inline in the platform modules.

## Known limitations & resolutions

| Limitation | Resolution |
|---|---|
| No global clipboard-change broadcast | `AddClipboardFormatListener` + message-only window thread |
| Clipboard lock contention | bounded retries (5×15ms), never spin |
| `CF_HDROP` paths go stale | store paths + metadata only; preview re-stats, shows "missing" |
| One owner per global hotkey | detect conflict, surface a rebind hint in Settings |
| Tray needs a window + loop | dedicated thread with `HWND_MESSAGE` parent |
| `CF_BITMAP` is device-dependent | prefer `CF_DIB`/`CF_DIBV5`; skip HBITMAP conversion |
| Delayed rendering / ownership | `SetClipboardData` with HGLOBAL transfers ownership; no owner HWND |

## License

**MIT** — see [LICENSE](LICENSE).

`SPDX-License-Identifier: MIT` · Copyright (c) 2026 Manuel.

Permission is granted to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, provided the copyright notice
and this permission notice are included in all copies or substantial portions.
The Software is provided "AS IS", without warranty of any kind. See the full
text in [LICENSE](LICENSE) for the complete terms.

Third-party dependencies are used under their own licenses (see `Cargo.lock`
and each crate's repository). ClipVault itself adds no additional restrictions.
