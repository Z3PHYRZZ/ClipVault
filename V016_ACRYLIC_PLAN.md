# Plan: Real frosted glass via native Windows acrylic/Mica translucency

## Context

v0.1.5 shipped a *simulated* frosted look: alpha-tinted surfaces over an opaque
static gradient. The user asked "is there any other way to make it use blur like
glass or frosted?" and chose **Full acrylic translucency**. Research overturned my
earlier claim that Iced hides the HWND: `iced::window::run_with_handle(id, f)`
(`iced_runtime-0.13.2/src/window.rs:398`, re-exported by `iced-0.13.1/src/window.rs:8`)
hands the app a `raw_window_handle::WindowHandle`, whose `RawWindowHandle::Win32`
variant exposes `Win32WindowHandle.hwnd` (`raw-window-handle-0.6.2/src/windows.rs:52`).
With that HWND, DWM backdrop APIs become reachable. `iced::window::Settings`
also has a `transparent: bool` field (`iced_core-0.13.2/src/window/settings.rs:56`)
that `iced_winit-0.13.0/src/conversion.rs:34` forwards to winit via
`.with_transparent(...)`.

Intended outcome: the desktop genuinely shows through the window with an OS-level
blur (acrylic on Win10/11, Mica on Win11), while text remains fully readable —
reconciling "full translucency" with the original spec's "readability never
compromised by background bleed-through". The v0.1.5 specular 1px borders,
diffused shadows, crisp pills, hover elevation, and two-color palette are
**retained and refined**, not replaced — this plan adds the missing real blur
layer beneath them. Delivered as a new **v0.1.6** tag, leaving v0.1.5 stable.

## Recommended approach

**One native backdrop module + a transparent root + an opaque content scrim.**

1. **Add dependencies** in `Cargo.toml`: `raw-window-handle = "0.6.2"` (exact
   version already in `Cargo.lock:3441`, so no new download) and enable the
   `Win32_Graphics_Dwm` feature on the existing `windows = "0.62.2"` dep.
   No network/telemetry crates.

2. **New `src/platform/backdrop.rs`** — all Win32 code stays behind
   `#[cfg(windows)]` with a `cfg(not(windows))` no-op stub, matching the existing
   pattern in `src/platform/singleton.rs` / `source.rs`.
   - `pub fn apply(hwnd: isize, dark: bool) -> Result<BackdropKind, String>`
   - Extract `HWND` from the handle in the caller, call this with the raw value.
   - **Windows 11**: `DwmSetWindowAttribute(hwnd,
     DWMWA_SYSTEMBACKDROP_TYPE (38), DWMSBT_MAINWINDOW (2), size_of::<i32>())`.
     Also `DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE (20), …)`
     so Mica matches the app theme. These constants exist in
     `windows-0.62.2/.../Dwm/mod.rs:237,241,190`.
   - **Windows 10 / fallback**: `SetWindowCompositionAttribute` is NOT in
     windows-rs (grep confirmed). Declare it manually via
     `windows::Win32::System::LibraryLoader::GetProcAddress` on `user32.dll`
     with the well-known `ACCENT_POLICY { AccentState=4 (ACRYLIC_BLUR_BEHIND),
     AccentFlags=2, GradientColor, AnimationId }` + `WINCOMPATTRDATA { Attrib=19
     (WCA_ACCENT_POLICY), pvData, cbData }` layout. If `GetProcAddress` or the
     call fails, fall back to `DwmEnableBlurBehindWindow` with `DWM_BLURBEHIND
     { dwFlags=1 (DWM_BB_ENABLE), fEnable=true, hRgnBlur=HRGN(0), .. }`
     (`windows-0.62.2/.../Dwm/mod.rs:18,257`). All failures return `Err`, never
     panic — the app keeps the current frosted look.
   - Return `BackdropKind { Mica | Acrylic | BlurBehind | None }` for logging and
     for the status bar ("Mica enabled" / "Frosted (software)").

3. **Get the HWND once at startup**: extend `Message` with
   `ApplyBackdrop` and fire it from `initialize()` in `src/main.rs` (currently
   returns `(AppState, Task::none())`). The handler in `src/app/update.rs` uses
   the existing `iced::window::get_oldest()` (`iced_runtime window.rs:253`) then
   `iced::window::run_with_handle(id, |h| …)` chained with `Task::then`:
   ```
   iced::window::get_oldest().then(|maybe_id| match maybe_id {
       Some(id) => iced::window::run_with_handle(id, |h| {
           // h.as_raw() -> RawWindowHandle::Win32(w).hwnd.get() -> isize
           // call backdrop::apply(hwnd, is_dark) ; send result via a Message
       }),
       None => Task::none(),
   })
   ```
   `run_with_handle`'s callback must be `Send + 'static` and returns `T` — have
   it return the `BackdropKind`, mapped back into `Message::BackdropApplied(kind)`
   so `AppState` can show status. Re-apply on theme change: `Message::SetTheme`
   already exists in `src/app/update.rs`; dispatch the same task there with the
   new `dark` value. Reuse `helpers` — no new window plumbing.

4. **Make the window transparent** in `src/main.rs`: set
   `transparent: true` in the existing `iced::window::Settings { … }` literal.
   Keep `decorations` default; document that winit transparency + DWM requires
   the composition path.

5. **Transparent root, opaque content** — the readability reconciliation:
   - `src/app/view.rs`: the root container currently draws
     `theme::ambient_background(t)` (an opaque gradient, lines 71–86). Replace
     with a **fully transparent** root style (`background: None`). Keep the
     existing `panel_style`/`glass_surface` styles on children.
   - `src/gui/theme.rs`: make the *chrome* (sidebar, search bar, toolbar, status
     bar) near-clear so the blurred desktop reads through — lower
     `GLASS_PANEL_*` alpha from ~0.78-0.80 to ~0.35-0.45. Keep the **content
     scrim opaque**: entry cards and preview pane must NOT go near-clear;
     raise `GLASS_CARD_*` back toward ~0.92-0.95 (still tinted CREAM/INK) so
     text, timestamps, source labels, and command previews keep >= 4.5:1
     contrast regardless of what's behind the window. `button_text()`'s existing
     composite-over-base logic must be re-checked against the new darker/lighter
     desktop-unknown base — keep the conservative readable_text() fallbacks.
   - `src/contracts.rs`: adjust the frosted token alphas (they are the single
     source of truth, already used by `theme.rs`). No new hues.
   - Keep `ambient_background()` available but unused by default as a
     software fallback when `BackdropKind::None` — best of both worlds: if DWM
     is unavailable (Win10 without acrylic, RDP, etc.) the root falls back to
     the gradient via an `AppState.backdrop` flag checked in `view.rs`.

6. **Performance / constraints**: DWM blur is compositor-side (GPU, not our
   widgets) — zero per-frame cost, no animated blur, no `backdrop-filter`-style
   work in Iced. No new runtime deps beyond a version already in the lockfile.
   All `windows-rs` calls `#[cfg(windows)]`-gated; `cargo test` stays headless.
   No `unwrap`/`expect` in runtime paths; graceful fallback on every Win32
   failure.

## Critical files

| File | Change |
|---|---|
| `Cargo.toml`, `Cargo.lock` | `raw-window-handle = "0.6.2"`, `Win32_Graphics_Dwm` feature; version bump to `0.1.6` (keep `winres = "0.1.12"`) |
| `src/platform/backdrop.rs` | NEW — DWM Mica / acrylic fallback, cfg-gated, `BackdropKind` |
| `src/platform/mod.rs` | `pub mod backdrop;` |
| `src/main.rs` | `transparent: true` in `window::Settings`; `initialize()` fires `Message::ApplyBackdrop`; version resource `0.1.6.0` |
| `src/app/message.rs` | `ApplyBackdrop`, `BackdropApplied(BackdropKind)` |
| `src/app/state.rs`, `src/workers/supervisor.rs`, `src/main.rs` fallback | `backdrop: BackdropKind` field in all 3 `AppState` constructors (same pattern as `window_hidden`) |
| `src/app/update.rs` | `get_oldest().then(run_with_handle(...))` task, re-apply on `SetTheme`, handle `BackdropApplied` |
| `src/app/view.rs` | transparent root; gradient fallback when `backdrop == None` |
| `src/gui/theme.rs`, `src/contracts.rs` | near-clear chrome alphas, opaque content scrim, contrast-safe text |
| `installer/ClipVault.iss`, `build.rs`, `installer/build-installer.bat` | `0.1.6` / `ClipVault-0.1.6-Setup.exe` |
| `CHANGELOG.md`, `docs/RELEASE_NOTES.md`, `README.md`, `ARCHITECTURE.md`, `SUMMARY.md`, `V016_ACRYLIC_PLAN.md` | docs; correct the earlier "Iced hides HWND" limitation claim |

Representative paths only — the version bump repeats across the same doc set as
prior releases (`docs/INSTALL.md`, `PACKAGING.md`, `UI_REWORK_SPEC.md` untouched).

## Reuse (do not re-invent)

- `iced::window::get_oldest()` and `iced::window::run_with_handle()` — existing
  public API, same `Task::then` pattern already used by `window_task()` in
  `src/app/update.rs:72`.
- `src/platform/PlatformError`-style cfg gating from `singleton.rs` / `source.rs`.
- `theme.rs` `button_text()`, `specular_border()`, `glass_surface()`,
  `ambient_background()` — extend, don't duplicate.
- `contracts.rs` `alpha()` helper and all `GLASS_*` / `SPECULAR_*` tokens.
- `AppState`'s 3-constructor pattern (state.rs / supervisor.rs / main.rs).
- Existing gate + release scripts: `installer/build-installer.bat`, ISCC,
  `gh release create`, PowerShell 5.1 `;` syntax (no `&&`).

## Verification (end-to-end)

1. `cargo fmt --check`, `cargo check --all-targets`,
   `cargo clippy --all-targets -- -D warnings`, `cargo test` (expect **49**
   tests, no new headless test can exercise DWM — add a unit test only for the
   `ACCENT_POLICY`/`WINCOMPATTRDATA` byte sizes and `BackdropKind` mapping).
2. `cargo build --release`; confirm PE subsystem 2 and version `0.1.6.0`.
3. **Manual visual smoke on the actual Windows 11 desktop** (this is the real
   proof, cannot be done headlessly): launch the exe, confirm the desktop blurs
   behind the sidebar/search bar, confirm all text/timestamps/source labels are
   still crisp; toggle Light/Dark in Settings and confirm Mica dark mode follows;
   switch to a busy/high-contrast wallpaper and re-check readability; resize the
   window and scroll history to confirm no flicker or blur cost.
4. `BackdropKind` logged via `tracing::info!` (no clipboard content) — read
   `%LOCALAPPDATA%\ClipVault\clipvault.log` to confirm which path was taken
   (Mica / Acrylic / BlurBehind / None) and that fallback never crashed.
5. Rebuild icon + installer, silent install / run / uninstall (0 / stays up / 0).
6. Commit + push, create `v0.1.6` GitHub Release with both assets, verify local
   sha256 matches GitHub API digests; run all six release tags still live.
7. Update `SUMMARY.md` and append the session to the AI-Brain vault with
   wikilinks (footer-anchored edit).

## Honest limitations to report

- Mica is **Windows 11 only**; Win10 uses experimental acrylic via an
  undocumented `user32` export, else `DwmEnableBlurBehindWindow`, else the
  existing gradient. The chosen "full translucency" varies by OS.
- A transparent winit window + DWM acrylic can be disabled under RDP / remote
  sessions / certain GPU drivers — the `None` fallback keeps the app usable.
- Content cards stay near-opaque by design; "full acrylic" applies to the window
  chrome, not to text-bearing surfaces, per the readability requirement.
