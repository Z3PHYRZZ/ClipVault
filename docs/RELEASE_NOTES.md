# ClipVault v0.1.5 — Frosted glass theme

A visual refinement release. New GitHub Release tag `v0.1.5`; v0.1.4 (organization
tools) stays stable. Same private, offline Windows clipboard manager — no network,
no telemetry, no console window. Palette unchanged: Cream `#F2E9E4` + Ink `#22223B`.

## What changed

A clean, modern frosted / Mica-inspired aesthetic, translated from CSS intent to
Iced 0.13 primitives (the app has no DOM and no CSS `backdrop-filter`):

- **Soft ambient gradient base** sits behind every UI layer — a static 3-stop
  linear gradient with no blur, giving depth without visual clutter.
- **Layered frosted surfaces**: sidebar, top search bar, and selection toolbar use
  a deeper tinted glass (~78-80% opacity) with a 20px diffused shadow to anchor
  navigation; entry cards, preview pane, and status bar use a softer ~64-66% glass
  with a 14px shadow.
- **1px specular borders**: a soft white highlight (0.40 light / 0.08 dark) plus
  contrasting hairlines keep overlapping frosted layers separated.
- **Crisp semi-transparent pills** for buttons, inputs, and dropdowns blend into
  the frosted containers.
- **Hover elevation**: shadow lift + border emphasis + subtle tint shift
  (equivalent to `translateY(-1px)`), message-driven, with no animated blur.
- **Readability first**: all text, timestamps, source labels, and command previews
  stay high contrast — verified >= 6.28:1 on buttons and >= 11.7:1 on glass
  surfaces in both themes (WCAG AA is 4.5:1).
- **Performance**: no compositor blur filter; only static gradients, constant
  alpha tints, and fixed-radius shadows. Scrolling and window resize stay smooth.

## Honest limitation

True OS-level Mica / Acrylic translucency is **not** applied. Iced 0.13 does not
expose the HWND (`iced_winit` keeps it internal), so `DwmSetWindowAttribute` would
require patching the GUI framework or an unreliable title-based `FindWindowW` race.
Mica is also Windows 11 only. The ambient gradient + layered alpha-tinted surfaces
are the framework-supported approximation and keep Windows 10 compatibility.
Design translation is documented in `V015_FROSTED_PLAN.md`.

## Assets

| Asset | What it is |
|---|---|
| `ClipVault-0.1.5-Setup.exe` | **Recommended.** Per-user installer (Inno Setup 6). No admin rights, Start Menu + optional desktop shortcut, optional launch-at-startup, clean uninstall that keeps your history. |
| `clipvault.exe` | Portable single-file executable. Run from any folder, no install. |

Windows 10/11 x64 only. No .NET, no VC++ redistributable, no network needed.
GUI-subsystem binary — **no console window**; diagnostics go to
`%LOCALAPPDATA%\ClipVault\clipvault.log`.

## Install

Double-click `ClipVault-0.1.5-Setup.exe`. If SmartScreen warns about an unsigned
binary, click **More info > Run anyway** (built from source in this repo, not code-signed).
Quit any running ClipVault (tray > Quit) before replacing an older install.
Databases upgrade automatically (up to schema v5) with all rows preserved.

Full docs: [docs/index.md](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/index.md)
· [Install guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/INSTALL.md)
· [User guide](https://github.com/Z3PHYRZZ/ClipVault/blob/main/docs/USER_GUIDE.md)
· [Changelog](https://github.com/Z3PHYRZZ/ClipVault/blob/main/CHANGELOG.md)

## Verification

- `cargo fmt --check`, `cargo check --all-targets`,
  `cargo clippy --all-targets -- -D warnings` — all clean
- `cargo test` — **49 passed / 0 failed** (33 unit + 10 db + 1 migration + 5 org)
- Release exe `0.1.5.0`, PE subsystem 2 (GUI)
- Installer rebuilt; silent install / run / uninstall verified
- Only two chromatic hex definitions in source; frost tokens are alpha/tint
  derivatives, no new hues
- Contrast verified numerically in both themes across the whole gradient

MIT licensed. Built by Manuel.
