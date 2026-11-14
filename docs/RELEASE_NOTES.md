# ClipVault v0.1.4 — Organization tools

A new release tag. v0.1.3 remains stable; this build adds day-to-day history
management. Same private, offline Windows clipboard manager (no network, no
telemetry, no console window). Palette stays Cream `#F2E9E4` + Ink `#22223B`.

## What's new

- **Tags / labels** — tag any entry with comma-separated labels, see `#tag` chips
  in the history list, edit them in the preview panel, and filter by tag from the
  search bar. Tags are normalized (trimmed, deduped, sorted) and stored locally in
  SQLite (schema v5).
- **Bulk select + multi-delete** — a checkbox per row, "Select all" / "Clear",
  "Delete selected" with a confirmation dialog, and one-click Pin / Favorite for
  the whole selection. Bulk operations run in a single transaction.
- **Portable JSON backup** — export your history (with images inlined as base64)
  to `%LOCALAPPDATA%\ClipVault\clipvault-export.json`, and import it back on this
  or another machine. Import is dedup-safe, so re-importing never duplicates rows.
  A 50 MB cap keeps large image histories from exhausting memory. No cloud.
- **Per-type retention** — set separate max-entry caps for text, images, and files
  in Settings (0 = use the global cap). Pinned and favorite entries are always
  kept. Older `settings.json` files load cleanly thanks to serde defaults.
- **List density** — switch between Comfortable and Compact row spacing from
  Settings or the status bar; the choice persists.

## Assets

| Asset | What it is |
|---|---|
| `ClipVault-0.1.4-Setup.exe` | **Recommended.** Per-user installer (Inno Setup 6). No admin rights, Start Menu + optional desktop shortcut, optional launch-at-startup, clean uninstall that keeps your history. |
| `clipvault.exe` | Portable single-file executable. Run from any folder, no install. |

Windows 10/11 x64 only. No .NET, no VC++ redistributable, no network needed.
GUI-subsystem binary — **no console window**; diagnostics go to
`%LOCALAPPDATA%\ClipVault\clipvault.log`.

## Install

Double-click `ClipVault-0.1.4-Setup.exe`. If SmartScreen warns about an unsigned
binary, click **More info > Run anyway** (built from source in this repo, not code-signed).
Quit any running ClipVault (tray > Quit) before replacing an older install.
Databases from v0.1.3 upgrade automatically (v4 -> v5) with all rows preserved.

Full docs: [docs/index.md](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/index.md)
· [Install guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/INSTALL.md)
· [User guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/USER_GUIDE.md)
· [Changelog](https://github.com/Z3PHYRZZ/ClipVault/blob/main/CHANGELOG.md)

## Verification

- `cargo fmt --check`, `cargo check --all-targets`,
  `cargo clippy --all-targets -- -D warnings` — all clean
- `cargo test` — **49 passed / 0 failed** (33 unit + 10 db + 1 migration + 5 org)
- Release exe `0.1.4.0`, PE subsystem 2 (GUI)
- Installer rebuilt; silent install / run / uninstall verified
- Only two chromatic hex definitions in source; no unrelated accent colors
- New: v4->v5 migration test with row preservation, tag/bulk/export-import tests

MIT licensed. Built by Manuel.
