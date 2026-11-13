# ClipVault v0.1.1

Palette refresh release. **No functional changes** — same private Windows clipboard
manager, now wearing the five-color
[coolors.co set](https://coolors.co/palette/22223b-4a4e69-9a8c98-c9ada7-f2e9e4).

## What changed

The entire color surface was replaced: GUI theme (light + dark), buttons, badges,
borders, shadows, selected states, the tray icon, the app icon, and all documentation.
Only these five hexes are used (alpha variants of them for hairlines/shadows):

| Role | Hex | Use |
|---|---|---|
| Cream | `#F2E9E4` | light background, cards, light text on dark |
| Rose | `#C9ADA7` | secondary panels, buttons, active state, selection |
| Mauve | `#9A8C98` | secondary text, icons, borders, quiet buttons |
| Slate | `#4A4E69` | dark surfaces, header accents |
| Ink | `#22223B` | dark-mode background, primary text, outlines |

- **Light theme:** Cream bg, Rose panels, Cream cards, Rose interactive, Ink type.
- **Dark theme:** Ink bg, Slate surfaces, Cream/Rose accents, Cream text —
  complementary, not inverted.
- `src/contracts.rs` exports `CREAM/ROSE/MAUVE/SLATE/INK` (+ RGB tuples) with
  backward-compatible aliases, so every module resolves to the new colors without
  API churn. `src/gui/theme.rs` is fully token-driven — no hardcoded RGB remains.
  Button text is INK on light / CREAM on dark for contrast.
- Tray/app icon redrawn: Ink rounded square, Cream clipboard card, Slate header,
  Rose "V" + clip tab, Mauve outline.
- Version bumped to **0.1.1** in `Cargo.toml`, `build.rs` (FileVersion/ProductVersion),
  Inno Setup script, installer filename, and all docs. `CHANGELOG.md` has the full
  [0.1.1] entry; [0.1.0] history is preserved.

## Assets

| Asset | What it is |
|---|---|
| `ClipVault-0.1.1-Setup.exe` | **Recommended.** Per-user installer (Inno Setup 6). No admin rights, Start Menu + optional desktop shortcut, optional launch-at-startup, clean uninstall that keeps your history. |
| `clipvault.exe` | Portable single-file executable. Run from any folder, no install. |

Windows 10/11 x64 only. No .NET, no VC++ redistributable, no network needed.
GUI-subsystem binary — **no console window**; diagnostics go to
`%LOCALAPPDATA%\ClipVault\clipvault.log`.

## Install

Double-click `ClipVault-0.1.1-Setup.exe`. If SmartScreen warns about an unsigned
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
- Release exe version `0.1.1.0`, PE subsystem 2 (GUI)
- Installer rebuilt; silent install / run / uninstall verified (0 / stays up / 0)
- Exactly 5 palette hex definitions in source; no unrelated accent colors

MIT licensed. Built by Manuel.
