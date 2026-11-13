# ClipVault v0.1.3

Two-color palette release. **No functional changes** — same private Windows
clipboard manager, now using only `#F2E9E4` and `#22223B`.

## What changed

The entire color surface was replaced: GUI theme (light + dark), buttons, badges,
borders, shadows, selected states, the tray icon, the app icon, and all docs.
Only these two colors are used as accents (plus neutral white/charcoal for text
contrast, and alpha variants of the two for hairlines/shadows):

| Role | Hex | Use |
|---|---|---|
| Cream | `#F2E9E4` | light background, cards, light text on dark, selection |
| Ink | `#22223B` | panels, buttons, active state, borders, dark surfaces, text |

- **Light theme:** Cream background, Ink panels, Ink text.
- **Dark theme:** Ink ground, Cream accents, Cream text — complementary, not inverted.
- `src/contracts.rs` exports `CREAM`/`INK` (+ RGB tuples, `WHITE`/`CHARCOAL`
  neutrals, `alpha()` helper) with backward-compatible aliases, so every module
  resolves to the two colors without API churn. `src/gui/theme.rs` is fully
  token-driven — no hardcoded RGB remains; button text picks contrast per surface.
- Tray/app icon redrawn with only the two brand colors: Ink rounded square,
  Cream clipboard card, Ink header + "V" mark, Cream outline.
- Version bumped to **0.1.3** in `Cargo.toml`, `build.rs`
  (FileVersion/ProductVersion), Inno Setup script, installer filename, and docs.
  `CHANGELOG.md` has the [0.1.3] entry; [0.1.2], [0.1.1], [0.1.0] preserved.

## What's new in this build (same v0.1.3 tag)

- **UI rework** — bigger readable fonts (28/21/16/14/12), roomier padding and
  radii, larger button hit areas with clear active/hover/pressed states, and
  **message-driven hover animation**: history cards and sidebar items lift their
  shadow by 3px, widen the border to 2px and blend the background on pointer
  enter/exit. Text contrast is chosen from the actual background, so nothing is
  invisible in either theme.
- **Global show/hide hotkey now works** — the configurable combo and the tray
  "Show" action minimize/restore the window (`window::minimize` + `gain_focus`)
  instead of being a no-op. `start_minimized` remains a documented gap (Iced 0.13
  has no true start-hidden-in-tray primitive).
- **Photo preview** — image entries show a lazy-loaded thumbnail directly in the
  history list, and the preview panel renders the full photo (300px, `Contain`)
  with its pixel dimensions. No GUI-thread file reads.
- **Copy provenance** — every capture records `source_app` (the process that owned
  the clipboard, e.g. `chrome.exe`) and `source_url` (the web page URL from the
  browser's `CF_HTML` `SourceURL:` header, or a bare http(s) link in the text).
  Best-effort; shown in the history subtitle and a SOURCE section in the preview.
  Persisted in SQLite schema v4.

## Assets

| Asset | What it is |
|---|---|
| `ClipVault-0.1.3-Setup.exe` | **Recommended.** Per-user installer (Inno Setup 6). No admin rights, Start Menu + optional desktop shortcut, optional launch-at-startup, clean uninstall that keeps your history. |
| `clipvault.exe` | Portable single-file executable. Run from any folder, no install. |

Windows 10/11 x64 only. No .NET, no VC++ redistributable, no network needed.
GUI-subsystem binary — **no console window**; diagnostics go to
`%LOCALAPPDATA%\ClipVault\clipvault.log`.

## Install

Double-click `ClipVault-0.1.3-Setup.exe`. If SmartScreen warns about an unsigned
binary, click **More info > Run anyway** (built from source in this repo, not code-signed).
Quit any running ClipVault (tray > Quit) before replacing an older install.

Full docs: [docs/index.md](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/index.md)
· [Install guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/INSTALL.md)
· [User guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/USER_GUIDE.md)
· [Changelog](https://github.com/Z3PHYRZZ/ClipVault/blob/main/CHANGELOG.md)

## Verification

- `cargo check` 0 errors / 0 warnings · `cargo fmt --check` clean ·
  `cargo clippy --all-targets -- -D warnings` 0 warnings
- `cargo test` **40 passed / 0 failed** (29 unit + 11 integration)
- Release exe version `0.1.3.0`, PE subsystem 2 (GUI)
- Installer rebuilt; silent install / run / uninstall verified
- Exactly 2 chromatic hex definitions in source; no unrelated accent colors

MIT licensed. Built by Manuel.
