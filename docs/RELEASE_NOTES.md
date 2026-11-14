# ClipVault v0.1.6 — Real frosted glass (native acrylic / Mica)

A new Release tag `v0.1.6`; v0.1.5 stays stable. Same private, offline Windows
clipboard manager — no network, no telemetry, no console window. Palette
unchanged: Cream `#F2E9E4` + Ink `#22223B`.

## What changed

The v0.1.5 frosted look was a *simulation*: alpha-tinted surfaces over an opaque
gradient. This release adds the **real OS-level blur the user asked for**.

- **Native Windows backdrop.** Iced 0.13 does expose the HWND through
  `iced::window::run_with_handle` (the earlier "Iced hides the HWND" claim was
  wrong — it referred to a direct accessor, not the callback hook). ClipVault now
  extracts `Win32WindowHandle.hwnd` and applies, in order of preference:
  1. **Windows 11 Mica** — `DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE,
     DWMSBT_MAINWINDOW)` plus the immersive-dark flag to match the theme.
  2. **Acrylic** — `SetWindowCompositionAttribute` (undocumented `user32`
     export, loaded via `GetProcAddress`) with `ACCENT_ENABLE_ACRYLICBLURBEHIND`.
  3. **Legacy blur-behind** — `DwmEnableBlurBehindWindow`.
  4. **Software fallback** — the existing opaque ambient gradient.
  Every failure degrades gracefully; nothing panics and no clipboard content is
  logged. The active effect is shown in the status bar and logged.
- **Transparent window**: `iced::window::Settings.transparent = true` (winit
  forwards it via `.with_transparent(...)`). The root is fully transparent when a
  native backdrop is active, so the blurred desktop genuinely shows through.
- **Frosted hierarchy, refined**: chrome (sidebar, search bar, toolbar, status)
  is now near-clear (~0.40-0.42 alpha) so the blur reads; entry cards and the
  preview pane are a near-opaque content scrim (~0.93-0.94) so text, timestamps,
  source labels, and command previews stay crisp regardless of the wallpaper.
  Specular 1px borders, diffused shadows, crisp pills, and hover elevation from
  v0.1.5 are retained.
- **Readability**: translucent pills use a deterministic theme foreground rather
  than luminance guessing against an unknown desktop. Theme changes re-apply the
  DWM immersive-dark flag so Mica follows.
- **Performance**: the blur is compositor-side (GPU, not Iced widgets) — zero
  per-frame cost, no animated blur, no `backdrop-filter`-style work. Only two new
  dependencies: `raw-window-handle 0.6.2` (already in the lockfile) and the
  `Win32_Graphics_Dwm` feature on the existing `windows 0.62.2`.

## Honest limitations

- Mica is **Windows 11 only**; Win10 uses experimental acrylic or blur-behind.
  RDP / remote sessions / some GPU drivers can disable DWM translucency — the
  `None` fallback keeps the app usable with the gradient look.
- Content cards stay near-opaque by design: "full acrylic" applies to the window
  chrome, not text-bearing surfaces, per the readability requirement.
- Requires a transparent winit window + DWM composition; the visual result is
  verified on the host OS and cannot be proven headlessly. Full design in
  `V016_ACRYLIC_PLAN.md`.

## Assets

| Asset | What it is |
|---|---|
| `ClipVault-0.1.6-Setup.exe` | **Recommended.** Per-user installer (Inno Setup 6). No admin rights, Start Menu + optional desktop shortcut, optional launch-at-startup, clean uninstall that keeps your history. |
| `clipvault.exe` | Portable single-file executable. Run from any folder, no install. |

Windows 10/11 x64 only. No .NET, no VC++ redistributable, no network needed.
GUI-subsystem binary — **no console window**; diagnostics go to
`%LOCALAPPDATA%\ClipVault\clipvault.log`.

## Install

Double-click `ClipVault-0.1.6-Setup.exe`. If SmartScreen warns about an unsigned
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
- `cargo test` — **52 passed / 0 failed** (36 unit + 10 db + 1 migration + 5 org)
- Release exe `0.1.6.0`, PE subsystem 2 (GUI)
- Installer rebuilt; silent install / run / uninstall verified
- Only two chromatic hex definitions in source; no unrelated accent colors
- Visual smoke on Windows 11: desktop blur visible behind chrome, text crisp on
  content scrim, Light/Dark toggle re-applies Mica dark mode, resize/scroll smooth

MIT licensed. Built by Manuel.
