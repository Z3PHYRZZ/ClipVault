# ClipVault v0.1.2

Two-color palette release. **No functional changes** — same private Windows
clipboard manager, now using only `#FFCAD4` and `#9D8189`.

## What changed

The entire color surface was replaced: GUI theme (light + dark), buttons, badges,
borders, shadows, selected states, the tray icon, the app icon, and all docs.
Only these two colors are used as accents (plus neutral white/charcoal for text
contrast, and alpha variants of the two for hairlines/shadows):

| Role | Hex | Use |
|---|---|---|
| Pastel Pink | `#FFCAD4` | light background, cards, light text on dark, selection |
| Muted Mauve | `#9D8189` | panels, buttons, active state, borders, dark surfaces, text |

- **Light theme:** Pink background, Mauve panels, charcoal text.
- **Dark theme:** Mauve ground, Pink accents, Pink text — complementary, not inverted.
- `src/contracts.rs` exports `PINK`/`MAUVE` (+ RGB tuples, `WHITE`/`CHARCOAL`
  neutrals, `alpha()` helper) with backward-compatible aliases, so every module
  resolves to the two colors without API churn. `src/gui/theme.rs` is fully
  token-driven — no hardcoded RGB remains; button text picks contrast per surface.
- Tray/app icon redrawn with only the two brand colors: Mauve rounded square,
  Pink clipboard card, Mauve header + "V" mark, Pink outline.
- Version bumped to **0.1.2** in `Cargo.toml`, `build.rs`
  (FileVersion/ProductVersion), Inno Setup script, installer filename, and docs.
  `CHANGELOG.md` has the [0.1.2] entry; [0.1.1] and [0.1.0] history preserved.

## Assets

| Asset | What it is |
|---|---|
| `ClipVault-0.1.2-Setup.exe` | **Recommended.** Per-user installer (Inno Setup 6). No admin rights, Start Menu + optional desktop shortcut, optional launch-at-startup, clean uninstall that keeps your history. |
| `clipvault.exe` | Portable single-file executable. Run from any folder, no install. |

Windows 10/11 x64 only. No .NET, no VC++ redistributable, no network needed.
GUI-subsystem binary — **no console window**; diagnostics go to
`%LOCALAPPDATA%\ClipVault\clipvault.log`.

## Install

Double-click `ClipVault-0.1.2-Setup.exe`. If SmartScreen warns about an unsigned
binary, click **More info > Run anyway** (built from source in this repo, not code-signed).
Quit any running ClipVault (tray > Quit) before replacing an older install.

Full docs: [docs/index.md](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/index.md)
· [Install guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/INSTALL.md)
· [User guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/USER_GUIDE.md)
· [Changelog](https://github.com/Z3PHYRZZ/ClipVault/blob/main/CHANGELOG.md)

## Verification

- `cargo check` 0 errors / 0 warnings · `cargo fmt --check` clean ·
  `cargo clippy --all-targets -- -D warnings` 0 warnings
- `cargo test` **35 passed / 0 failed**
- Release exe version `0.1.2.0`, PE subsystem 2 (GUI)
- Installer rebuilt; silent install / run / uninstall verified
- Exactly 2 chromatic hex definitions in source; no unrelated accent colors

MIT licensed. Built by Manuel.
