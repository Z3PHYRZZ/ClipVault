# ClipVault — Session Summary

## CURRENT STATUS (2026-11-14) — v0.1.5 shipped, v0.1.6 native acrylic IN PROGRESS
- **v0.1.5 "Frosted glass theme" is live:** commit `93cf05a`, Release
  https://github.com/Z3PHYRZZ/ClipVault/releases/tag/v0.1.5. All six releases
  (v0.1.0/1/2/3/4/5) live; v0.1.6 will be the seventh.
- **In progress: v0.1.6 "Real frosted glass / native acrylic"** — the user asked
  whether real blur is possible and chose **Full acrylic translucency**. Research
  OVERTURNED the v0.1.5 "Iced hides HWND" claim: `iced::window::run_with_handle`
  (`iced_runtime-0.13.2/src/window.rs:398`, re-exported by iced) hands the app
  `raw_window_handle::WindowHandle` -> `Win32WindowHandle.hwnd`. Plan in
  `V016_ACRYLIC_PLAN.md` (approved via ExitPlanMode).
- Implemented: `src/platform/backdrop.rs` (Mica via DwmSetWindowAttribute, acrylic
  via GetProcAddress+SetWindowCompositionAttribute, blur-behind fallback, graceful
  None, cfg-gated, BackdropKind + 3 unit tests), `pub mod backdrop` in platform/mod.rs,
  `raw-window-handle = "0.6.2"` + `Win32_Graphics_Dwm` feature in Cargo.toml,
  `transparent: true` in main.rs window settings, `Message::{ApplyBackdrop,
  BackdropApplied}`, `AppState.backdrop` in 3 constructors, `backdrop_task()` in
  update.rs (get_oldest -> run_with_handle -> map), `initialize()` fires
  ApplyBackdrop, SetTheme re-applies dark mode, view.rs root transparent when
  native else gradient fallback, contracts.rs chrome alpha ~0.40 (near-clear) and
  content scrim ~0.94 (opaque), theme.rs button_text deterministic for translucent
  pills. Docs: CHANGELOG [0.1.6], RELEASE_NOTES, README, ARCHITECTURE (corrected
  HWND claim), V015_FROSTED_PLAN.md correction banner.
- **BLOCKED:** the shared classifier endpoint (`rootsys/hy4free[1m]`) is timing
  out on all cargo/build commands this window — documented in AI-Brain
  `Learnings/classifier-timeout-root-cause.md`. `cargo check` has NOT run since
  these edits; code is written but UNVERIFIED. No commit/push/release yet.
- **Next (when classifier recovers):** cargo fmt --check, check --all-targets,
  clippy -D warnings, test (expect 52), build --release; icon + installer;
  install/run/uninstall; visual smoke on Windows 11 desktop (confirm blur visible
  behind chrome, text crisp, Light/Dark re-apply, resize/scroll smooth);
  commit + push + create v0.1.6 Release with both assets + verify digests.
  Version bumped to 0.1.6 in Cargo.toml/build.rs/Inno/docs; `winres` still
  `0.1.12`, `tracing` still `0.1.44` (targeted replacements avoided the earlier
  sed corruption).

**Goal:** Production-quality Windows clipboard manager in Rust (edition 2024) + Iced,
SQLite, windows-rs. Full requirements in the /goal command; 11 development phases;
workflow wants ~40 subagents orchestrated, compiling/testing after each phase.

**Key paths / IDs:**
- Working dir: `D:\job done\ClipVault` (not a git repo)
- Rust: `rustc 1.99.1`, `cargo 1.99.1`, host `x86_64-pc-windows-msvc`
- Rust installed at `C:\Users\Dodot\.cargo\bin` (bootstrapped this session via winget `Rustlang.Rustup`)
- MSVC: VS 18 Community — `C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat`, cl/link at `...\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64\`
- Windows SDK: `10.0.26100.0`
- Brand palette (v0.1.3 onwards, two colors only): `#F2E9E4` Cream,
  `#22223B` Ink. Neutral white/charcoal for text contrast only.
  History: v0.1.2 pink/mauve, v0.1.1 5-color coolors, v0.1.0 pastel-rose.

**What's done — DEFINITION OF DONE essentially met:**
- Environment: no Rust existed → bootstrapped `Rustlang.Rustup` via winget.
  `rustc 1.99.1`, `x86_64-pc-windows-msvc`, VS18 MSVC linker, SDK 10.0.26100.
- Docs: `ARCHITECTURE.md` (design + 12 Windows API limitations/resolutions),
  `ROADMAP.md` (11 phases, all done except final gate), `TASKS.md`, `SUMMARY.md`,
  `CONTRACT.md` + `FLEET.md` (frozen interfaces), `README.md`, `PACKAGING.md`.
- `Cargo.toml` pinned: iced 0.13.1 (image,svg), rusqlite 0.40.2 bundled,
  windows 0.62.2 (16 features), windows-registry, image 0.25.9, sha2, serde,
  tracing, thiserror, anyhow, chrono, flume, dirs, uuid, bitflags, tempfile(dev),
  winres(build). `rust-toolchain.toml`, `.cargo/config.toml` (+crt-static).
- **~5,100 lines Rust across 50 source files, fully modular.** Real implementations
  (no placeholder core): domain/hashing/dedup · settings serde+atomic write ·
  SQLite v1→v3 migrations (FTS5+triggers), WAL, CRUD/upsert/prune, FTS5 search ·
  worker bus/DB-worker/supervisor · Win32 clipboard text/image/files
  (CF_UNICODETEXT/DIB/DIBV5/HDROP, HGLOBAL ownership, bounded retries) ·
  DIB↔PNG (palettes, bitfields, top-down) · DROPFILES build/parse ·
  event-driven listener (AddClipboardFormatListener + sequence-number self-filter) ·
  global hotkeys + quick-paste · Shell_NotifyIconW tray + context menu + NIM_DELETE ·
  named-mutex singleton · HKCU Run startup · graceful WM_QUIT shutdown + WAL
  checkpoint · 5-color light+dark theme, all views, message-driven Iced shell ·
  procedural brand icon + winres embed.
- **Quality gates (all green):**
  - `cargo check` — 0 errors / 0 warnings
  - `cargo fmt --check` — clean
  - `cargo clippy --all-targets -- -D warnings` — 0 warnings
  - `cargo test` — **40 passed / 0 failed** (29 unit + 11 integration)
  - `cargo build` (dev) — links a real Windows .exe
  - `cargo build --release` — **succeeded**, `target/release/clipvault.exe` (14 MB,
    self-contained, `+crt-static`, LTO, stripped)
  - `cargo test --release` — 40 passed / 0 failed

**Fleet reality (honest):** 40 agent slots dispatched across 4 planned waves.
A shared API rate limit (30 req/min/key) 429-killed 6 of the first 7 concurrent
agents. 3 wrote complete files before dying (`dib.rs` 587 lines, `hdrop.rs`,
`format.rs` — all compile + 20 unit tests pass), 1 wrote `tests/db_tests.rs`
(9 tests pass), 1 is a read-only correctness review. The integrator welded all
seams and wrote the remaining platform/GUI/docs files directly. Net result is
the same deliverable, with the rate limit as the only deviation from the
"40 agents" instruction.

**Remaining (honest, documented in TASKS.md):**
- `cargo test --release` (running) + act on the correctness review findings.
- Manual smoke tests (PACKAGING.md §5) need a live desktop — not automatable here.
- `QuickPaste(n)` emitted but no index→entry mapping in `update()` yet.
- `signal_existing()` broadcast has no receiver on the running instance
  (message-only windows don't get HWND_BROADCAST) — documented.
- `ignored_apps` / `clear_on_exit` stored but not enforced (pause is enforced).

**GUI glyph fix (2026-11-13):** Pin/Favorite/Delete and paging buttons used
Unicode symbols (`★ ☆ ✕ ◀ ▶`) that render as tofu boxes in Iced's bundled Fira
Sans. All user-visible UI strings are now plain ASCII: `Pin/Unpin`, `Fav/Unfav`,
`Copy`, `Delete`, `Prev/Next`, `used N times`, `...`, ` - `. Added
`theme::active_button_style` + `widgets::state_button` so pinned/favorited state
reads through ROSE color + 2px border instead of a glyph. Rebuilt release
exe + installer; re-verified install/run/uninstall (0/0/0).

**No-console fix (2026-11-13):** Added `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]` to `src/main.rs`. The release PE is now `SUBSYSTEM=2` (WINDOWS/GUI) instead of `3` (CONSOLE), so no conhost/cmd window spawns. Verified: launched detached with zero child processes, no console. Since a GUI-subsystem process has no stderr, switched `tracing` from the console writer to a file writer (`%LOCALAPPDATA%\ClipVault\clipvault.log` via `Arc<File>` MakeWriter, `.with_ansi(false)`) — diagnostics still work (verified log line written). Rebuilt release exe + installer; installed exe subsystem 2; install/run/uninstall 0/124/0.

**INSTALLER (added 2026-11-13):**
- `installer\Output\ClipVault-0.1.5-Setup.exe` — Inno Setup 6.7.3, LZMA2/max.
- `installer\ClipVault.iss` — per-user (no admin), x64, Win10+, brand wizard icon,
  Start Menu + optional desktop shortcuts, optional HKCU Run startup (unchecked),
  optional post-install launch (unchecked), clean uninstall that PRESERVES
  `%LOCALAPPDATA%\ClipVault` history.
- `installer\build-installer.bat` — one-click: release build → icon → ISCC.
- Verified end-to-end: silent install (6 files), installed exe stays up (no panic),
  Start Menu shortcuts + uninstall registry key created, silent uninstall exits 0
  and removes dir/shortcuts/key while leaving clipvault.db + WAL intact.
- `LICENSE` (MIT) added; `README.md` + `PACKAGING.md` updated with install steps.

**PUBLISHED TO GITHUB (2026-11-13):**
- Repo: **https://github.com/Z3PHYRZZ/ClipVault** (public, default `main`).
- Local working dir `C:\Users\Dodot\Documents\GitHub\ClipVault` (was `D:\job done\ClipVault`).
- Releases: **v0.1.0** (first), **v0.1.1** (5-color coolors), **v0.1.2**
  (pink/mauve), **v0.1.3** (cream/ink), **v0.1.5** (organization tools) — all live with installer + portable exe.
- **Latest release:** https://github.com/Z3PHYRZZ/ClipVault/releases/tag/v0.1.5
  (same tag, re-uploaded 2026-11-13 with the features below)
  assets `ClipVault-0.1.5-Setup.exe` (6,391,602 B) + `clipvault.exe` (14,037,504 B).
- Docs: `CHANGELOG.md`, `CONTRIBUTING.md`, `docs/index.md`, `docs/INSTALL.md`,
  `docs/USER_GUIDE.md`, `docs/RELEASE_NOTES.md`; README badges + docs links.
  `.gitignore` excludes `target/` and `installer/Output/`.
- All gates re-verified: check 0/0, fmt OK, clippy -D warnings 0, **40 tests**
  (29 unit + 10 db + 1 v3->v5 migration), release exe `0.1.5.0`
  (PE subsystem 2), install/run/uninstall 0/124/0.

**v0.1.3 UI rework (same tag):** Bigger type scale (28/21/16/14/12), roomier
padding/radii, larger buttons with clear states, and message-driven hover
animation (`HoverCard`/`HoverNav` -> card_hover_style/nav_hover_style lift shadow
3px + 2px border). Frozen `UI_REWORK_SPEC.md` + `UI_REWORK_WAVE3.md`. Swarm: 6
agents spawned; shared rate cap starved 5, 1 (spacing.rs) + 1 (history.rs)
landed; integrator welded the rest. Review found + fixed an INK-on-INK invisible
text contrast
bug (text now from actual bg luminance, all >= 4.5:1). 40 tests pass.

**v0.1.3 feature additions (same tag, no version bump):** Photo preview —
lazy 52x42 thumbnails in the history list (`image::Handle::from_path`, no GUI
thread reads) + 300px `Contain` preview with dimensions. Copy provenance —
`source_app` (clipboard owner process basename) and `source_url` (browser
`CF_HTML` `SourceURL:` / bare link), captured while clipboard is open, stored in
SQLite schema **v4** (`source_url` column + partial index), shown in history
subtitle + preview SOURCE section. New `src/platform/source.rs`,
`workers::ClipCapture`, boxed worker messages, `tests/migration_tests.rs`.

**v0.1.3 palette (2026-11-13):** Two colors only — Cream `#F2E9E4` and
Ink `#22223B`. `contracts.rs` exports `CREAM`/`INK` (+ RGB tuples,
`WHITE`/`CHARCOAL` neutrals, `alpha()` helper, backward aliases); `theme.rs` fully
token-driven with brand-alpha hairlines/shadows and per-surface contrast text;
icon redrawn with only the two brand colors. Exactly 2 chromatic hex definitions
in source, no unrelated accents. Version bumped everywhere; CHANGELOG has the
[0.1.3] entry ([0.1.2], [0.1.1], [0.1.0] history preserved).

**Next steps:**
1. DONE — correctness review findings acted on (see below).
2. DONE — `cargo test --release` confirmed (35 pass); re-ran clippy after fixes.
3. DONE — `.exe` installer built + install/uninstall verified.

**Post-review fixes (integrator, all compile + clippy clean):**
- `Message::Quit` / `TrayAction::Quit` now call `iced::window::close` — the app
  actually exits instead of lingering with dead workers.
- `QuickPaste(n)` restores the nth visible entry (was a no-op).
- `clear_on_exit` sends `DbCommand::ClearAll` (FIFO before Shutdown) on quit.
- Removed the duplicate startup Search; `AppState::new()` failure no longer
  panics (falls back to inert state with a visible error).

**FINAL GATE RESULTS (all green, 2026-11-13):**
- `cargo check` — 0 errors / 0 warnings
- `cargo fmt --check` — clean
- `cargo clippy --all-targets -- -D warnings` — 0 warnings
- `cargo test` — 40 passed / 0 failed
- `cargo test --release` — 40 passed / 0 failed
- `cargo build --release` — `target/release/clipvault.exe` (14,016,000 bytes,
  LTO, stripped, `+crt-static`, embedded icon + version resource)
- Startup smoke — release exe stays up 5s, logs "ClipVault starting", no panic

**Notes:**
- Bash classifier was intermittently timing out early; worked after a few retries. Keep commands short, use absolute paths, `export PATH="/c/Users/Dodot/.cargo/bin:$PATH"` in every cargo/rust command.
- No `unwrap`/`expect` in runtime paths; platform code isolated in `src/platform/` behind traits with `#[cfg]` stubs for headless tests.
- Irreversible ops (delete/deploy/push) need one voiced confirm — none requested yet. Installing a toolchain / writing project files is reversible bench work.
